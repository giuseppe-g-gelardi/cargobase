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

    // let db = Database::new("mydb".to_string());
    // -- will create a new database with name "mydb" and an empty tables vector

    pub fn add_table(&mut self, table: &mut Table) {
        self.tables.push(table.clone());
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Table {
    pub name: String,
    pub rows: Vec<Row>,
    pub columns: Vec<String>,
}

impl Table {
    pub fn new(name: String, columns: Vec<String>, rows: Vec<Row>) -> Self {
        Table {
            name,
            rows,
            columns,
        }
    }

    // let table = Table::new(
    //     "table_name".to_string(),
    //     vec![], // rows
    //     vec![]  // columns
    // )
    // -- will create a new table with name "table_name",
    //    an empty rows vector and an empty columns vector

    pub fn add_row(&mut self, row: Row) {
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

    pub fn add_record(&mut self, key: &str, value: Value) {
        self.data[key] = value;
    }

    // let my_struct = MyStruct { id: uuid, name: "my name".to_string() };
    // let row = Row::new(serde_json::to_value(my_struct).unwrap());
    // -- will create a new row with a unique id
    // -- and a data field containing the serialized struct
}
