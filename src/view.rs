use crate::{Database, Table};

pub struct View<'a> {
    database: &'a Database,
}

impl<'a> View<'a> {
    /// Create a new `View` instance
    pub fn new(database: &'a Database) -> Self {
        View { database }
    }

    /// Display all tables in the database
    pub fn all_tables(&self) {
        println!("Database: {}", self.database.name);

        for table in self.database.tables.values() {
            self.display_table(table);
        }
    }

    /// Display a specific table by name
    pub fn single_table(&self, table_name: &str) {
        if let Some(table) = self.database.tables.get(table_name) {
            self.display_table(table);
        } else {
            println!("Table '{}' not found in the database.", table_name);
        }
    }

    /// Display a single table
    fn display_table(&self, table: &Table) {
        println!("\nTable: {}", table.name);

        if table.columns.0.is_empty() {
            println!("No columns defined for table '{}'.", table.name);
            return;
        }

        // Get column names and determine maximum width for each column
        let column_names: Vec<&str> = table
            .columns
            .0
            .iter()
            .map(|col| col.name.as_str())
            .collect();
        let mut column_widths: Vec<usize> = column_names.iter().map(|name| name.len()).collect();

        // Adjust column widths based on the content of each row
        for row in table.rows.values() {
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
        for row in table.rows.values() {
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
