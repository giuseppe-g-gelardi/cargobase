pub mod core;

use crate::Table;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Database {
    pub name: String,
    pub file_name: PathBuf,
    pub tables: HashMap<String, Table>,
}
