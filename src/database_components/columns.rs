use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_reflection::{ContainerFormat, Named, Tracer, TracerConfig};

use crate::DatabaseError;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Column {
    pub name: String,
    pub required: bool,
}

impl Column {
    pub fn new(name: &str, required: bool) -> Self {
        Column {
            name: name.to_string(),
            required,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Columns(pub Vec<Column>);

impl Columns {
    // define new columns
    pub fn new(columns: Vec<Column>) -> Self {
        Columns(columns)
    }

    // remove required ??
    pub fn from_struct<T>(required: bool) -> Self
    where
        T: Serialize + DeserializeOwned + Default,
    {
        // Initialize the reflection tracer
        let mut tracer = Tracer::new(TracerConfig::default());
        tracer
            .trace_simple_type::<T>()
            .expect("Failed to trace struct");

        // Retrieve the container format for the struct
        let registry = tracer.registry().expect("Failed to get registry");
        let type_name = std::any::type_name::<T>().split("::").last().unwrap();
        let container = registry
            .get(type_name)
            .expect("Struct not found in registry");

        // Extract fields in declared order
        let columns = if let ContainerFormat::Struct(fields) = container {
            fields
                .iter()
                .map(|Named { name, .. }| Column::new(name, required))
                .collect()
        } else {
            vec![]
        };

        Columns(columns)
    }

    // validate the columns
    pub fn validate(&self, row_data: Value) -> Result<(), DatabaseError> {
        if let Value::Object(data) = row_data {
            for column in &self.0 {
                if column.required && !data.contains_key(&column.name) {
                    let error_message = format!("Column '{}' is required.", column.name);
                    tracing::error!("{}", error_message);
                    return Err(DatabaseError::ColumnRequiredError(error_message));
                }
            }

            for key in data.keys() {
                if !self.0.iter().any(|col| col.name == *key) {
                    let error_message = format!("Column '{}' is not valid.", key);
                    tracing::error!("{}", error_message);
                    return Err(DatabaseError::InvalidData(error_message));
                }
            }
            Ok(())
        } else {
            tracing::error!("Invalid row data.");
            Err(DatabaseError::InvalidData("Invalid row data.".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tracing_test::traced_test;

    use super::*;

    #[test]
    fn test_column_new() {
        let column = Column::new("name", true);
        assert_eq!(column.name, "name");
        assert!(column.required);
    }

    #[test]
    fn test_columns_new() {
        let columns = Columns::new(vec![Column::new("name", true), Column::new("age", false)]);
        assert_eq!(columns.0.len(), 2);
        assert_eq!(columns.0[0].name, "name");
        assert!(columns.0[0].required);
        assert_eq!(columns.0[1].name, "age");
        assert!(!columns.0[1].required);
    }

    #[test]
    fn test_validate_valid_row() {
        let columns = Columns(vec![
            Column::new("id", true),
            Column::new("name", true),
            Column::new("email", false),
        ]);

        let row_data = json!({
            "id": "123",
            "name": "John Doe",
            "email": "john.doe@example.com"
        });

        let result = columns.validate(row_data);
        assert!(result.is_ok());
    }

    #[traced_test]
    #[test]
    fn test_validate_missing_required_column() {
        let columns = Columns(vec![
            Column::new("id", true),
            Column::new("name", true),
            Column::new("email", false),
        ]);

        let row_data = json!({
            "id": "123",
            "email": "john.doe@example.com"
        });

        let result = columns.validate(row_data);

        // Assert that an error is returned
        assert!(result.is_err());

        // Verify the specific log message
        assert!(
            logs_contain("Column 'name' is required."),
            "Expected log message for missing required column not found."
        );
    }

    #[traced_test]
    #[test]
    fn test_validate_invalid_column() {
        let columns = Columns(vec![
            Column::new("id", true),
            Column::new("name", true),
            Column::new("email", false),
        ]);

        let row_data = json!({
            "id": "123",
            "name": "John Doe",
            "phone": "123-456-7890" // Invalid column
        });

        let result = columns.validate(row_data);

        // Assert that an error is returned
        assert!(result.is_err());

        // Verify the specific log message
        assert!(
            logs_contain("Column 'phone' is not valid."),
            "Expected log message for invalid column not found."
        );
    }

    #[traced_test]
    #[test]
    fn test_validate_invalid_row_type() {
        let columns = Columns(vec![
            Column::new("id", true),
            Column::new("name", true),
            Column::new("email", false),
        ]);

        let row_data = json!([
            {
                "id": "123",
                "name": "John Doe",
                "email": "john.doe@example.com"
            }
        ]); // Invalid type (array instead of object)

        let result = columns.validate(row_data);

        // Assert that an error is returned
        assert!(result.is_err());

        // Verify the specific log message
        assert!(
            logs_contain("Invalid row data."),
            "Expected log message for invalid row data not found."
        );
    }

    #[test]
    fn test_validate_optional_column_missing() {
        let columns = Columns(vec![
            Column::new("id", true),
            Column::new("name", true),
            Column::new("email", false), // Optional column
        ]);

        let row_data = json!({
            "id": "123",
            "name": "John Doe"
        });

        let result = columns.validate(row_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_columns_from_struct() {
        #[derive(Serialize, Deserialize, Default)]
        struct TestData {
            id: String,
            first_name: String,
            last_name: String,
            email: String,
            age: u8,
            bio: String,
            location: String,
        }
        let columns = Columns::from_struct::<TestData>(true);
        // testing the order of the Columns as the from_struct method will
        // preserve the order of the struct fields as they are declared
        assert_eq!(columns.0.len(), 7);
        assert_eq!(columns.0[0].name.to_string(), "id".to_string());
        assert_eq!(columns.0[1].name.to_string(), "first_name".to_string());
        assert_eq!(columns.0[2].name.to_string(), "last_name".to_string());
        assert_eq!(columns.0[3].name.to_string(), "email".to_string());
        assert_eq!(columns.0[4].name.to_string(), "age".to_string());
        assert_eq!(columns.0[5].name.to_string(), "bio".to_string());
        assert_eq!(columns.0[6].name.to_string(), "location".to_string());
    }
}
