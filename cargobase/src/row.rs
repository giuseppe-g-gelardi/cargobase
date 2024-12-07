use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Row {
    pub _id: String, // uuid v4
    pub data: Value,
}

impl Row {
    pub fn new(data: Value) -> Self {
        let _id = Uuid::new_v4().to_string();
        Row { _id, data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_new() {
        let row = Row::new(serde_json::json!({"name": "John Doe", "age": 30}));
        assert_eq!(row.data["name"], "John Doe");
        assert_eq!(row.data["age"], 30);
    }

    #[test]
    fn test_row_id() {
        let row = Row::new(serde_json::json!({"name": "John Doe", "age": 30}));
        assert_eq!(Uuid::parse_str(&row._id).is_ok(), true);
    }

    #[test]
    fn test_row_id_unique() {
        let row1 = Row::new(serde_json::json!({"name": "John Doe", "age": 30}));
        let row2 = Row::new(serde_json::json!({"name": "Jane Doe", "age": 25}));
        assert_ne!(row1._id, row2._id);
    }

    #[test]
    fn test_row_id_length() {
        // test the length of the uuid v4
        let row = Row::new(serde_json::json!({"name": "John Doe", "age": 30}));
        assert_eq!(row._id.len(), 36);
    }
}
