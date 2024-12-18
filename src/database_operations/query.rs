use crate::{Database, Operation, Query};

impl Database {
    pub fn add_row(&mut self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Create,
            update_data: None,
            row_data: None,
        }
    }

    pub fn get_rows(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Read,
            update_data: None,
            row_data: None,
        }
    }

    pub fn get_single(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Read,
            update_data: None,
            row_data: None,
        }
    }

    pub fn delete_single(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Delete,
            update_data: None,
            row_data: None,
        }
    }

    pub fn update_row(&self) -> Query {
        Query {
            db_file_name: self.file_name.clone(),
            table_name: None,
            operation: Operation::Update,
            update_data: None,
            row_data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_add_row() {
        let mut db = Database {
            name: "test_db".to_string(),
            file_name: "test_db.json".into(),
            tables: HashMap::new(),
        };

        let query = db.add_row();
        assert_eq!(query.operation, Operation::Create);
    }

    #[tokio::test]
    async fn test_get_rows() {
        let db = Database {
            name: "test_db".to_string(),
            file_name: "test_db.json".into(),
            tables: HashMap::new(),
        };

        let query = db.get_rows();
        assert_eq!(query.operation, Operation::Read);
    }

    #[tokio::test]
    async fn test_get_single() {
        let db = Database {
            name: "test_db".to_string(),
            file_name: "test_db.json".into(),
            tables: HashMap::new(),
        };

        let query = db.get_single();
        assert_eq!(query.operation, Operation::Read);
    }

    #[tokio::test]
    async fn test_delete_single() {
        let db = Database {
            name: "test_db".to_string(),
            file_name: "test_db.json".into(),
            tables: HashMap::new(),
        };

        let query = db.delete_single();
        assert_eq!(query.operation, Operation::Delete);
    }

    #[tokio::test]
    async fn test_update_row() {
        let db = Database {
            name: "test_db".to_string(),
            file_name: "test_db.json".into(),
            tables: HashMap::new(),
        };

        let query = db.update_row();
        assert_eq!(query.operation, Operation::Update);
    }
}
