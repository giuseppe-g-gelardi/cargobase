use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Database {
    pub name: String, // get name from command line args?
    pub tables: Vec<Table>,
}

impl Database {
    pub fn new(name: String) -> Self {
        Database {
            name,
            tables: Vec::new(),
        }
    }

    pub fn add_table(&mut self, table: &mut Table) {
        self.tables.push(table.clone());
    }

    pub fn save_to_file(&self, file_name: &str) -> Result<(), std::io::Error> {
        let json_data = serde_json::to_string_pretty(&self)?;
        std::fs::write(file_name, json_data)?;
        Ok(())
    }

    // pub fn load_from_file(file_name: &str) -> Result<Self, std::io::Error> {
    //     let json_data = std::fs::read_to_string(file_name)?;
    //     let db: Database = serde_json::from_str(&json_data)?;
    //     Ok(db)
    // }

    pub fn add_row(
        &mut self,
        table_name: &str,
        data: Value,
        file_name: &str,
    ) -> Result<(), String> {
        if let Some(table) = self.tables.iter_mut().find(|t| t.name == table_name) {
            if let Err(e) = table.add_row(data) {
                return Err(format!("Failed to add row: {}", e));
            }

            if let Err(e) = self.save_to_file(file_name) {
                return Err(format!("Failed to save to file: {}", e));
            }

            Ok(())
        } else {
            Err(format!("Table {} not found", table_name))
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Table {
    pub name: String,
    pub rows: Vec<Row>,
    pub columns: Columns,
}

impl Table {
    pub fn new(name: String, columns: Columns) -> Self {
        Table {
            name,
            rows: Vec::new(),
            columns,
        }
    }

    pub fn add_row(&mut self, data: Value) -> Result<(), String> {
        self.columns.validate(&data)?;
        let row = Row::new(data);
        self.rows.push(row);
        Ok(())
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
    // pub data_type: DataType,
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

    pub fn validate(&self, data: &Value) -> Result<(), String> {
        if let Some(obj) = data.as_object() {
            for column in &self.0 {
                if column.required && !obj.contains_key(&column.name) {
                    return Err(format!("Missing required column: {}", column.name));
                }
            }
            Ok(())
        } else {
            Err("Invalid data format: expected a JSON object.".to_string())
        }
    }
}
