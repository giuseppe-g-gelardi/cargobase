mod database;

use database::database::Database;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

fn main() {
    let db = Database::new();
    let new_file = append_to_file("cargobase.json".to_string(), &db);

    println!("db {:?}", new_file);
}

pub fn append_to_file(database_name: String, record: &Database) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .write(true)
        .open(database_name)?;

    let mut writer = BufWriter::new(file);
    let record_json = serde_json::to_string(&record)?;

    writeln!(writer, "{}", record_json)?;
    writer.flush()?;

    Ok(())
}
