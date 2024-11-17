use serde::{Deserialize, Serialize};

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
    pub fn new(columns: Vec<Column>) -> Self {
        Columns(columns)
    }
}
