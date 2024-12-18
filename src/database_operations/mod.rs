pub mod core;
pub mod io;
pub mod query;
pub mod utils;

use crate::Table;

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Database {
    pub(crate) name: String,
    pub(crate) file_name: PathBuf,
    pub(crate) tables: HashMap<String, Table>,
}

