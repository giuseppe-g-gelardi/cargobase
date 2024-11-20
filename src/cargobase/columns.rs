use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Column {
    pub(crate) name: String,
    pub(crate) required: bool,
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
    pub fn new(columns: Vec<Column>) -> Self {
        Columns(columns)
    }

    pub fn from_struct<T: Serialize + Default>(required: bool) -> Self {
        let value = json!(T::default());
        let columns = if let Value::Object(map) = value {
            map.keys().map(|key| Column::new(key, required)).collect()
        } else {
            vec![]
        };
        Columns(columns)
    }

    pub fn validate(&self, row_data: Value) -> Result<(), String> {
        if let Value::Object(data) = row_data {
            for column in &self.0 {
                if column.required && !data.contains_key(&column.name) {
                    return Err(format!("Column '{}' is required.", column.name));
                }
            }

            for key in data.keys() {
                if !self.0.iter().any(|col| col.name == *key) {
                    return Err(format!("Column '{}' is not valid.", key));
                }
            }
            Ok(())
        } else {
            Err("Invalid row data.".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_new() {
        let column = Column::new("name", true);
        assert_eq!(column.name, "name");
        assert_eq!(column.required, true);
    }

    #[test]
    fn test_columns_new() {
        let columns = Columns::new(vec![Column::new("name", true), Column::new("age", false)]);
        assert_eq!(columns.0.len(), 2);
        assert_eq!(columns.0[0].name, "name");
        assert_eq!(columns.0[0].required, true);
        assert_eq!(columns.0[1].name, "age");
        assert_eq!(columns.0[1].required, false);
    }

    #[test]
    fn test_columns_from_struct() {
        #[derive(Serialize, Deserialize, Default)]
        struct Test {
            name: String,
            age: String,
        }
        // the from_struct method will organize the columns in alphabetical order

        let columns = Columns::from_struct::<Test>(true);
        assert_eq!(columns.0.len(), 2);
        assert_eq!(columns.0[0].name.to_string(), "age".to_string());
        assert_eq!(columns.0[0].required, true);
        assert_eq!(columns.0[1].name.to_string(), "name".to_string());
        assert_eq!(columns.0[1].required, true);

        println!("generated columns: {:#?}", columns);
    }

    #[test]
    fn test_columns_from_struct_required_false() {
        #[derive(Serialize, Deserialize, Default)]
        struct Test {
            name: String,
            age: String,
        }
        // the from_struct method will organize the columns in alphabetical order

        let columns = Columns::from_struct::<Test>(false);
        assert_eq!(columns.0.len(), 2);
        assert_eq!(columns.0[0].name.to_string(), "age".to_string());
        assert_eq!(columns.0[0].required, false);
        assert_eq!(columns.0[1].name.to_string(), "name".to_string());
        assert_eq!(columns.0[1].required, false);

        println!("generated columns: {:#?}", columns);
    }
}
