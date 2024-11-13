use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Database {
    pub name: String, // get name from command line args
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

    pub fn add_row(&mut self, table_name: &str, data: Value) {
        if let Some(table) = self.tables.iter_mut().find(|t| t.name == table_name) {
            table.add_row(data);
        } else {
            println!("Table not found");
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Table {
    pub name: String,
    pub rows: Vec<Row>,
    pub columns: Vec<String>,
}

impl Table {
    pub fn new(name: String, columns: Vec<String>) -> Self {
        Table {
            name,
            rows: Vec::new(),
            columns,
        }
    }

    // pub fn add_row(&mut self, data: Value) {
    //     let row = Row::new(data);
    //     self.rows.push(row);
    // }

    pub fn add_row(&mut self, data: Value) {
        if let Some(obj) = data.as_object() {
            for column in &self.columns {
                if !obj.contains_key(column) {
                    println!("Missing required column: {}", column);
                    return;
                }
            }
        } else {
            println!("Invalid data format: expected a JSON object.");
            return;
        }
        let row = Row::new(data);
        self.rows.push(row);
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
