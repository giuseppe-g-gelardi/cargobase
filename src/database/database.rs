use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Database {
    pub name: String,
    pub file_name: String,
    pub tables: Vec<Table>,
}

impl Database {
    pub fn new(name: String) -> Self {
        let file_name = format!("{}.json", name);

        if std::path::Path::new(&file_name).exists() {
            // println!("Database {} already exists, Loading from file.", file_name);
            // let db = Database::load_from_file(&file_name).unwrap();
            // return db;
            return Database::load_from_file(&file_name).unwrap();
        } else {
            println!("Creating new database: {}", file_name);

            if let Err(e) = std::fs::write(&file_name, "{}") {
                eprintln!("Failed to create database file: {}", e);
            }
        }

        Database {
            name,
            file_name,
            tables: Vec::new(),
        }
    }

    pub fn add_table(&mut self, table: &mut Table) {
        table.set_file_name(self.file_name.clone());
        if self.tables.iter().any(|t| t.name == table.name) {
            println!("Table {} already exists, Skipping creation.", table.name);
        } else {
            self.tables.push(table.clone());
        }
    }

    fn save_to_file(&self) -> Result<(), std::io::Error> {
        let json_data = serde_json::to_string_pretty(&self)?;
        std::fs::write(&self.file_name, json_data)?;
        println!("Database saved to file: {}", self.file_name);
        Ok(())
    }

    fn load_from_file(file_name: &str) -> Result<Self, std::io::Error> {
        let json_data = std::fs::read_to_string(file_name)?;
        let db: Database = serde_json::from_str(&json_data)?;
        Ok(db)
    }

    fn get_table_mut(&mut self, table_name: &str) -> Option<&mut Table> {
        self.tables.iter_mut().find(|t| t.name == table_name)
    }

    pub fn get_rows(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
        }
    }

    pub fn get_single(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Query {
    db_file_name: String,
    table_name: Option<String>,
}

impl Query {
    pub fn from(mut self, table_name: &str) -> Self {
        self.table_name = Some(table_name.to_string());
        self
    }

    pub fn all<T: DeserializeOwned>(&self) -> Vec<T> {
        let db = Database::load_from_file(&self.db_file_name).unwrap_or_else(|e| {
            eprintln!("Failed to load database from file: {}", e);
            Database {
                name: String::new(),
                file_name: self.db_file_name.clone(),
                tables: Vec::new(),
            }
        });

        if let Some(table_name) = &self.table_name {
            if let Some(table) = db.tables.iter().find(|t| t.name == *table_name) {
                table
                    .rows
                    .iter()
                    .filter_map(|row| serde_json::from_value(row.data.clone()).ok())
                    .collect()
            } else {
                eprintln!("Table {} not found", table_name);
                Vec::new()
            }
        } else {
            eprintln!("Table name not provided");
            Vec::new()
        }
    }

    // pub fn where_eq<T: DeserializeOwned>(self, key: &str, value: &str) -> Option<T> {
    pub fn where_eq<T: DeserializeOwned>(self, key: &str, value: &str) -> Result<T, String> {
        // let db = Database::load_from_file(&self.db_file_name).unwrap_or_else(|e| {
        //     eprintln!("Failed to load database from file: {}", e);
        //     Database {
        //         name: String::new(),
        //         file_name: self.db_file_name.clone(),
        //         tables: Vec::new(),
        //     }
        // });
        let db = Database::load_from_file(&self.db_file_name)
            .map_err(|e| format!("Failed to load database from file: {}", e))?;

        if let Some(table_name) = &self.table_name {
            if let Some(table) = db.tables.iter().find(|t| t.name == *table_name) {
                for row in &table.rows {
                    if let Some(obj) = row.data.get(key) {
                        if obj.as_str() == Some(value) {
                            // return serde_json::from_value(row.data.clone()).ok();
                            return serde_json::from_value(row.data.clone())
                                .map_err(|e| format!("Deserialization error: {}", e));
                        }
                    }
                }
                // eprintln!("No matching row found");
                Err(format!("No matching row found"))
            } else {
                Err(format!("Table {} not found", table_name))
                // eprintln!("Table {} not found", table_name);
            }
        } else {
            // eprintln!("Table name not provided");
            Err(format!("Table name not provided"))
        }
        // None
    }

    // pub fn from<T: DeserializeOwned>(&self, table_name: &str) -> Vec<T> {
    //     let db = Database::load_from_file(&self.db_file_name).unwrap_or_else(|e| {
    //         eprintln!("Failed to load database from file: {}", e);
    //         Database {
    //             name: String::new(),
    //             file_name: self.db_file_name.clone(),
    //             tables: Vec::new(),
    //         }
    //     });
    //
    //     if let Some(table) = db.tables.iter().find(|t| t.name == table_name) {
    //         // table.rows.clone()
    //         table
    //             .rows
    //             .iter()
    //             .filter_map(|row| serde_json::from_value(row.data.clone()).ok())
    //             .collect()
    //     } else {
    //         eprintln!("Table {} not found", table_name);
    //         Vec::new()
    //     }
    // }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Table {
    pub name: String,
    pub rows: Vec<Row>,
    pub columns: Columns,
    pub file_name: Option<String>, // reference to the db file_name
}

impl Table {
    pub fn new(name: String, columns: Columns) -> Self {
        Table {
            name,
            rows: Vec::new(),
            columns,
            file_name: None,
        }
    }

    pub fn set_file_name(&mut self, file_name: String) {
        self.file_name = Some(file_name);
    }

    pub fn add_row(&mut self, db: &mut Database, data: Value) {
        if let Some(table) = db.get_table_mut(&self.name) {
            if data.is_array() {
                if let Some(array) = data.as_array() {
                    for item in array {
                        table.rows.push(Row::new(item.clone()))
                    }
                }
            } else {
                table.rows.push(Row::new(data))
            }
            let _ = db.save_to_file().map_err(|e| {
                println!("Failed to save to file: {}", e);
            });
        } else {
            println!("Table {} not found", self.name);
        }
    }
}

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
pub struct Columns(Vec<Column>);

impl Columns {
    pub fn new(columns: Vec<Column>) -> Self {
        Columns(columns)
    }

    // pub fn validate(&self, data: &Value) -> Result<(), String> {
    //     if let Some(obj) = data.as_object() {
    //         for column in &self.0 {
    //             if column.required && !obj.contains_key(&column.name) {
    //                 return Err(format!("Missing required column: {}", column.name));
    //             }
    //         }
    //         Ok(())
    //     } else {
    //         Err("Invalid data format: expected a JSON object.".to_string())
    //     }
    // }
}
