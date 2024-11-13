// use crate::database::database::Database;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::super::database::database::Database;

use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};

// pub fn append_to_file(database_name: String, record: &Database) -> Result<(), Box<dyn Error>> {
//     let file = OpenOptions::new()
//         .append(true)
//         .create(true)
//         .write(true)
//         .open(database_name)?;
//
//     let mut writer = BufWriter::new(file);
//     let record_json = serde_json::to_string(&record)?;
//
//     writeln!(writer, "{}", record_json)?;
//     writer.flush()?;
//
//     Ok(())
// }

// pub fn save_to_file<T: Serialize>(file_path: &str, data: &T) -> Result<(), Box<dyn Error>> {
pub fn save_to_file(file_path: &str, database: &Database) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file_path)?;

    // let data_json = serde_json::to_string_pretty(data)?;
    // let mut writer = BufWriter::new(file);
    // writer.write_all(data_json.as_bytes())?;
    // writer.flush()?;
    let mut writer = BufWriter::new(file);
    let database_json = serde_json::to_string_pretty(&database)?;
    writer.write_all(database_json.as_bytes())?;
    writer.flush()?;
    // serde_json::to_writer(writer, data)?;

    Ok(())
}

// pub fn load_from_file<T: DeserializeOwned>(file_path: &str) -> Result<T, Box<dyn Error>> {
pub fn load_from_file(file_path: &str) -> Result<Database, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let data = serde_json::from_reader(reader)?;
    Ok(data)
}

// pub fn save_to_file(database_name: &str, database: &Database) -> Result<(), Box<dyn Error>> {
// pub fn save_to_file<T: Serialize>(database: &Database<T>) -> Result<(), Box<dyn Error>> {
//     let db_name = &database.name;
//     let append_json = db_name.to_owned() + &".json".to_string();
//     // Open file for writing, truncate to overwrite existing content
//     let file = OpenOptions::new()
//         .write(true)
//         .create(true)
//         .truncate(true)
//         .open(append_json)?;
//
// //  .open(&database.name + ".json".to_string())?;
// // 32 +         .open(database.name + &".json".to_string())?;
//
//     // Serialize the database to JSON format
//     let database_json = serde_json::to_string_pretty(&database)?;
//
//     // Write the serialized JSON to file
//     let mut writer = BufWriter::new(file);
//     writer.write_all(database_json.as_bytes())?;
//     writer.flush()?;
//
//     Ok(())
// }

// pub fn load_from_file(database_name: &str) -> Result<Database, Box<dyn Error>> {
//     // Attempt to open the file, return an empty database if it doesn't exist
//     let file = match File::open(database_name) {
//         Ok(file) => file,
//         Err(_) => {
//             return Ok(Database {
//                 name: database_name.to_string(),
//                 tables: Vec::new(),
//             })
//         }
//     };
//
//     // Read the contents of the file
//     let mut content = String::new();
//     let mut reader = std::io::BufReader::new(file);
//     reader.read_to_string(&mut content)?;
//
//     // Deserialize the JSON content into a Database struct
//     let database: Database = serde_json::from_str(&content)?;
//
//     Ok(database)
// }
