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

    pub fn drop_table(&mut self, table_name: &str) -> Result<(), String> {
        let mut db = Database::load_from_file(&self.file_name)
            .map_err(|e| format!("Failed to laod database from file: {:?}", e))?;

        if let Some(index) = db.tables.iter().position(|t| t.name == table_name) {
            let removed_table = db.tables.remove(index);
            println!("Table {} dropped successfully", removed_table.name);
            db.save_to_file()
                .map_err(|e| format!("Failed to save database: {:?}", e))?;

            self.tables = db.tables;
            Ok(())
        } else {
            Err(format!("Table {} not found", table_name))
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
            delete: false,
        }
    }

    pub fn get_single(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            delete: false,
        }
    }

    pub fn delete_single(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            delete: true,
        }
    }

    pub fn view(&self) {
        println!("Database: {}", self.name);

        for table in &self.tables {
            println!("\nTable: {}", table.name);

            if table.columns.0.is_empty() {
                println!("No columns defined for table '{}'.", table.name);
                continue;
            }

            // Get column names and determine maximum width for each column
            let column_names: Vec<&str> = table
                .columns
                .0
                .iter()
                .map(|col| col.name.as_str())
                .collect();
            let mut column_widths: Vec<usize> =
                column_names.iter().map(|name| name.len()).collect();

            // Adjust column widths based on the content of each row
            for row in &table.rows {
                for (i, column) in table.columns.0.iter().enumerate() {
                    let value = row
                        .data
                        .get(&column.name)
                        .unwrap_or(&serde_json::Value::Null)
                        .to_string();
                    column_widths[i] = column_widths[i].max(value.len());
                }
            }

            // Print the header row
            let header: Vec<String> = column_names
                .iter()
                .enumerate()
                .map(|(i, &name)| format!("{:<width$}", name, width = column_widths[i]))
                .collect();
            println!("{}", header.join(" | "));

            // Print a separator line
            let separator: Vec<String> = column_widths
                .iter()
                .map(|&width| "-".repeat(width))
                .collect();
            println!("{}", separator.join("-+-"));

            // Print each row of data
            for row in &table.rows {
                let row_data: Vec<String> = table
                    .columns
                    .0
                    .iter()
                    .enumerate()
                    .map(|(i, column)| {
                        let value = row
                            .data
                            .get(&column.name)
                            .unwrap_or(&serde_json::Value::Null)
                            .to_string();
                        format!("{:<width$}", value, width = column_widths[i])
                    })
                    .collect();
                println!("{}", row_data.join(" | "));
            }
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Query {
    db_file_name: String,
    table_name: Option<String>,
    delete: bool,
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

    /// Fetch or delete a single row by a specific key-value pair
    pub fn where_eq<T: DeserializeOwned>(self, key: &str, value: &str) -> Result<T, String> {
        // Load the latest state of the database from the file
        let mut db = Database::load_from_file(&self.db_file_name)
            .map_err(|e| format!("Failed to load database from file: {}", e))?;

        if let Some(table_name) = &self.table_name {
            if let Some(table) = db.tables.iter_mut().find(|t| t.name.as_str() == table_name) {
                for i in 0..table.rows.len() {
                    let row = &table.rows[i];
                    if let Some(field_value) = row.data.get(key) {
                        if field_value.as_str() == Some(value) {
                            // Deserialize the matching record
                            let record: T = serde_json::from_value(row.data.clone())
                                .map_err(|e| format!("Deserialization error: {}", e))?;

                            // Check if the operation is "delete"
                            if self.delete {
                                table.rows.remove(i);
                                db.save_to_file()
                                    .map_err(|e| format!("Failed to save database: {}", e))?;
                                println!("Deleted record from table '{}'.", table_name);
                            }

                            // Return the found or deleted record
                            return Ok(record);
                        }
                    }
                }
                Err(format!(
                    "No matching record found where '{}' == '{}'.",
                    key, value
                ))
            } else {
                Err(format!("Table '{}' not found.", table_name))
            }
        } else {
            Err("Table name not specified.".to_string())
        }
    }
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

    fn validate_row(&self, data: &Value) -> Result<(), String> {
        if let Some(obj) = data.as_object() {
            for column in &self.columns.0 {
                if column.required && !obj.contains_key(&column.name) {
                    return Err(format!("Missing required column: {}", column.name));
                }
            }

            for key in obj.keys() {
                if !self.columns.0.iter().any(|col| col.name == *key) {
                    return Err(format!("Invalid column name: {}", key));
                }
            }
            Ok(())
        } else {
            Err("Invalid data format: expected a JSON object.".to_string())
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
}
