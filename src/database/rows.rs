use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Row {
    pub columns: Vec<String>,
    pub values: Vec<String>,
}

// impl Row {
//     pub fn new(columns: Vec<String>, values: Vec<String>) -> Self {
//         Row { columns, values }
//     }
// }
