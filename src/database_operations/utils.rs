use crate::{Database, Table};

impl Database {
    pub(crate) fn get_table_mut(&mut self, table_name: &str) -> Option<&mut Table> {
        tracing::debug!("looking for table: {}", table_name);
        let table = self.tables.get_mut(table_name);

        if table.is_some() {
            tracing::debug!("table found: {}", table_name);
        } else {
            tracing::error!("table not found: {}", table_name);
        }

        table
    }
}

#[cfg(test)]

mod tests {

    use crate::{setup_temp_db, Table};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
    struct TestData {
        id: String,
        name: String,
    }

    #[tokio::test]
    async fn test_get_table_mut() {
        let mut db = setup_temp_db().await;
        let test_columns = crate::Columns::from_struct::<TestData>(true);

        let mut table = Table::new("test_table_mut".to_string(), test_columns.clone());
        db.add_table(&mut table)
            .await
            .expect("failed to add test_table_mut");

        let table = db.get_table_mut("test_table_mut");
        assert!(table.is_some());
    }
}
