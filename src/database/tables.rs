use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Table {
    name: String,
    columns: Vec<String>,
}

// impl Table {
//     pub fn new(name: String, columns: Vec<String>) -> Self {
//         Table { name, columns }
//     }
// }
