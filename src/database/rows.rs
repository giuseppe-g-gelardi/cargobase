use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
