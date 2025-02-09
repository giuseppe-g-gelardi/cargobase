pub mod core;

use crate::Table;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Database {
    pub(crate) name: String,
    pub(crate) file_name: PathBuf,
    pub(crate) tables: HashMap<String, Table>,
}
