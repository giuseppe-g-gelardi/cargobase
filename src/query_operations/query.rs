use std::collections::HashMap;
use std::fmt;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::{Database, DatabaseError, Operation, Query, Row, Table};

impl Query {
    pub fn from(mut self, table_name: &str) -> Self {
        self.table_name = Some(table_name.to_string());
        self
    }

    pub fn data(mut self, data: Value) -> Self {
        // For bulk operations, data should go into row_data if it's an array
        if matches!(self.operation, Operation::BulkCreate) && data.is_array() {
            self.row_data = Some(data);
        } else {
            self.update_data = Some(data);
        }
        self
    }

    pub fn data_from_struct<T: Serialize>(mut self, data: T) -> Self {
        self.row_data = Some(serde_json::to_value(data).expect("Failed to serialize data"));
        self
    }

    /// For bulk operations - accepts Vec<T>
    pub fn data_many<T: Serialize>(mut self, data: Vec<T>) -> Self {
        self.row_data = Some(serde_json::to_value(data).expect("Failed to serialize data"));
        self
    }

    // Enhanced query builder methods
    pub fn where_gt(mut self, field: &str, value: impl Into<Value>) -> Self {
        self.add_condition(field, crate::ComparisonOp::Gt, value.into());
        self
    }

    pub fn where_lt(mut self, field: &str, value: impl Into<Value>) -> Self {
        self.add_condition(field, crate::ComparisonOp::Lt, value.into());
        self
    }

    pub fn where_gte(mut self, field: &str, value: impl Into<Value>) -> Self {
        self.add_condition(field, crate::ComparisonOp::Gte, value.into());
        self
    }

    pub fn where_lte(mut self, field: &str, value: impl Into<Value>) -> Self {
        self.add_condition(field, crate::ComparisonOp::Lte, value.into());
        self
    }

    pub fn where_ne(mut self, field: &str, value: impl Into<Value>) -> Self {
        self.add_condition(field, crate::ComparisonOp::Ne, value.into());
        self
    }

    /// Builder-style equality check (use with bulk operations)
    /// For single-row queries, use `where_eq().await` instead
    pub fn where_equals(mut self, field: &str, value: impl Into<Value>) -> Self {
        self.add_condition(field, crate::ComparisonOp::Eq, value.into());
        self
    }

    pub fn where_like(mut self, field: &str, pattern: impl Into<String>) -> Self {
        self.add_condition(field, crate::ComparisonOp::Like, Value::String(pattern.into()));
        self
    }

    pub fn where_in(mut self, field: &str, values: Vec<Value>) -> Self {
        self.add_condition(field, crate::ComparisonOp::In, Value::Array(values));
        self
    }

    pub fn where_not_in(mut self, field: &str, values: Vec<Value>) -> Self {
        self.add_condition(field, crate::ComparisonOp::NotIn, Value::Array(values));
        self
    }

    pub fn and(mut self) -> Self {
        // Start a new AND condition group
        if !self.conditions.is_empty() {
            self.conditions.push(crate::ConditionGroup::new(crate::LogicalOp::And));
        }
        self
    }

    pub fn or(mut self) -> Self {
        // Start a new OR condition group
        if !self.conditions.is_empty() {
            self.conditions.push(crate::ConditionGroup::new(crate::LogicalOp::Or));
        }
        self
    }

    pub fn order_by(mut self, field: impl Into<String>, order: crate::SortOrder) -> Self {
        self.order_by.push((field.into(), order));
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    // Helper method to add conditions to the appropriate group
    fn add_condition(&mut self, field: &str, operator: crate::ComparisonOp, value: Value) {
        let condition = crate::Condition::new(field, operator, value);
        
        if self.conditions.is_empty() {
            // First condition - create an AND group by default
            let mut group = crate::ConditionGroup::new(crate::LogicalOp::And);
            group.add_condition(condition);
            self.conditions.push(group);
        } else {
            // Add to the last group
            if let Some(last_group) = self.conditions.last_mut() {
                last_group.add_condition(condition);
            }
        }
    }

    // pub async fn where_eq<T: DeserializeOwned + Default>(
    pub async fn where_eq<T>(self, key: &str, value: &str) -> Result<Option<T>, DatabaseError>
    where
        T: DeserializeOwned + Default,
    {
        let mut db = Database::load_from_file(&self.db_file_name)
            .await
            .map_err(DatabaseError::LoadError)?;
        self.handle_where_eq(&mut db, key, value).await // Shared logic
    }

    // Shared logic for where_eq
    async fn handle_where_eq<T>(
        &self,
        db: &mut Database,
        key: &str,
        value: &str,
    ) -> Result<Option<T>, DatabaseError>
    where
        T: DeserializeOwned + Default,
    {
        let table_name = self
            .table_name
            .clone()
            .ok_or_else(|| DatabaseError::TableNotFound("Table name not specified.".to_string()))?;

        let table = db.tables.get_mut(&table_name).ok_or_else(|| {
            DatabaseError::TableNotFound(format!("Table '{}' not found.", table_name))
        })?;

        match self.operation {
            Operation::Read => self.execute_select(table, key, value),
            Operation::Update => {
                let result = self.execute_update(table, key, value);
                db.save_to_file().await.map_err(DatabaseError::SaveError)?;
                result
            }
            Operation::Delete => {
                let result = self.execute_delete(table, key, value);
                db.save_to_file().await.map_err(DatabaseError::SaveError)?;
                result
            }
            Operation::Create | Operation::BulkCreate | Operation::BulkUpdate | Operation::BulkDelete => {
                unreachable!("Bulk operations should use their specific execute methods")
            }
        }
    }

    #[must_use]
    pub async fn execute_add(self) -> Result<(), DatabaseError> {
        let mut db = Database::load_from_file(&self.db_file_name)
            .await
            .map_err(DatabaseError::LoadError)?;
        self.handle_execute_add_sync(&mut db).await // Shared logic
    }

    async fn handle_execute_add_sync(&self, db: &mut Database) -> Result<(), DatabaseError> {
        let table_name = self
            .table_name
            .clone()
            .ok_or_else(|| DatabaseError::InvalidData("Table name not specified.".to_string()))?;

        let table = db
            .tables
            .get_mut(&table_name)
            .ok_or_else(|| DatabaseError::TableNotFound(table_name.clone()))?;

        if let Some(row_data) = self.row_data.clone() {
            table.columns.validate(row_data.clone())?;

            if let Some(row_id) = row_data.get("id").and_then(|id| id.as_str()) {
                table.rows.insert(row_id.to_string(), Row::new(row_data));
            } else {
                return Err(DatabaseError::InvalidData(
                    "No 'id' field provided for the new row.".to_string(),
                ));
            }

            db.save_to_file().await.map_err(DatabaseError::SaveError)?;
            Ok(())
        } else {
            Err(DatabaseError::InvalidData(
                "No data provided for the new row.".to_string(),
            ))
        }
    }

    /// Execute bulk insert - returns count of inserted rows
    #[must_use]
    pub async fn execute_bulk_insert(self) -> Result<usize, DatabaseError> {
        let mut db = Database::load_from_file(&self.db_file_name)
            .await
            .map_err(DatabaseError::LoadError)?;

        let table_name = self
            .table_name
            .clone()
            .ok_or_else(|| DatabaseError::InvalidData("Table name not specified.".to_string()))?;

        let table = db
            .tables
            .get_mut(&table_name)
            .ok_or_else(|| DatabaseError::TableNotFound(table_name.clone()))?;

        let rows_data = self
            .row_data
            .clone()
            .ok_or_else(|| DatabaseError::InvalidData("No data provided for bulk insert.".to_string()))?;

        // Handle both array and single object
        let rows_array = match rows_data {
            Value::Array(arr) => arr,
            single_obj @ Value::Object(_) => vec![single_obj],
            _ => return Err(DatabaseError::InvalidData(
                "Bulk insert data must be an array or object.".to_string(),
            )),
        };

        let mut inserted_count = 0;
        let mut errors = Vec::new();

        for (idx, row_data) in rows_array.iter().enumerate() {
            // Validate each row
            match table.columns.validate(row_data.clone()) {
                Ok(()) => {
                    if let Some(row_id) = row_data.get("id").and_then(|id| id.as_str()) {
                        table.rows.insert(row_id.to_string(), Row::new(row_data.clone()));
                        inserted_count += 1;
                    } else {
                        errors.push(format!("Row {}: No 'id' field provided", idx));
                    }
                }
                Err(e) => {
                    errors.push(format!("Row {}: {}", idx, e));
                }
            }
        }

        // Save to file once after all inserts
        db.save_to_file().await.map_err(DatabaseError::SaveError)?;

        if !errors.is_empty() {
            tracing::warn!("Bulk insert completed with {} errors: {:?}", errors.len(), errors);
        }

        Ok(inserted_count)
    }

    /// Execute bulk update - returns count of updated rows
    #[must_use]
    pub async fn execute_bulk_update(self) -> Result<usize, DatabaseError> {
        let mut db = Database::load_from_file(&self.db_file_name)
            .await
            .map_err(DatabaseError::LoadError)?;

        let table_name = self
            .table_name
            .clone()
            .ok_or_else(|| DatabaseError::InvalidData("Table name not specified.".to_string()))?;

        let table = db
            .tables
            .get_mut(&table_name)
            .ok_or_else(|| DatabaseError::TableNotFound(table_name.clone()))?;

        let update_data = self
            .update_data
            .clone()
            .ok_or_else(|| DatabaseError::InvalidData("No update data provided.".to_string()))?;

        let mut updated_count = 0;

        // Get all rows that match conditions
        let mut rows_to_update: Vec<String> = Vec::new();
        for (row_id, row) in table.rows.iter() {
            if self.evaluate_conditions(&row.data) {
                rows_to_update.push(row_id.clone());
            }
        }

        // Update matching rows
        for row_id in rows_to_update {
            if let Some(row) = table.rows.get_mut(&row_id) {
                self.apply_update_to_row(row, &Some(update_data.clone()))?;
                updated_count += 1;
            }
        }

        if updated_count > 0 {
            db.save_to_file().await.map_err(DatabaseError::SaveError)?;
        }

        Ok(updated_count)
    }

    /// Execute bulk delete - returns count of deleted rows
    #[must_use]
    pub async fn execute_bulk_delete(self) -> Result<usize, DatabaseError> {
        let mut db = Database::load_from_file(&self.db_file_name)
            .await
            .map_err(DatabaseError::LoadError)?;

        let table_name = self
            .table_name
            .clone()
            .ok_or_else(|| DatabaseError::InvalidData("Table name not specified.".to_string()))?;

        let table = db
            .tables
            .get_mut(&table_name)
            .ok_or_else(|| DatabaseError::TableNotFound(table_name.clone()))?;

        // Find all rows that match conditions
        let mut rows_to_delete: Vec<String> = Vec::new();
        for (row_id, row) in table.rows.iter() {
            if self.evaluate_conditions(&row.data) {
                rows_to_delete.push(row_id.clone());
            }
        }

        // Delete matching rows
        let deleted_count = rows_to_delete.len();
        for row_id in rows_to_delete {
            table.rows.remove(&row_id);
        }

        if deleted_count > 0 {
            db.save_to_file().await.map_err(DatabaseError::SaveError)?;
        }

        Ok(deleted_count)
    }

    fn execute_select<T>(
        &self,
        table: &Table,
        key: &str,
        value: &str,
    ) -> Result<Option<T>, DatabaseError>
    where
        T: DeserializeOwned,
    {
        for row in table.rows.values() {
            if let Some(field_value) = row.data.get(key) {
                if field_value.as_str() == Some(value) {
                    return serde_json::from_value(row.data.clone())
                        .map(Some)
                        .map_err(|e| {
                            DatabaseError::InvalidData(format!("Deserialization error: {}", e))
                        });
                }
            }
        }
        Ok(None) // No matching record found
    }

    fn execute_update<T>(
        &self,
        table: &mut Table,
        key: &str,
        value: &str,
    ) -> Result<Option<T>, DatabaseError>
    where
        T: DeserializeOwned,
    {
        for row in table.rows.values_mut() {
            if let Some(field_value) = row.data.get(key) {
                if field_value.as_str() == Some(value) {
                    self.apply_update_to_row(row, &self.update_data)?;

                    tracing::info!("Record updated successfully.");
                    return self.deserialize_row(row);
                }
            }
        }
        Ok(None) // No matching record found
    }

    // Helper: Apply the update data to the row
    fn apply_update_to_row(
        &self,
        row: &mut Row,
        update_data: &Option<Value>,
    ) -> Result<(), DatabaseError> {
        let update_map = match update_data {
            Some(Value::Object(map)) => map,
            Some(_) => {
                return Err(DatabaseError::InvalidData(
                    "Invalid update data format.".to_string(),
                ))
            }
            None => {
                return Err(DatabaseError::InvalidData(
                    "No update data provided.".to_string(),
                ))
            }
        };

        let row_map = row.data.as_object_mut().ok_or_else(|| {
            DatabaseError::InvalidData("Row data is not a JSON object.".to_string())
        })?;

        for (k, v) in update_map.iter() {
            row_map.insert(k.clone(), v.clone());
        }

        Ok(())
    }

    // Helper: Deserialize the updated row
    fn deserialize_row<T>(&self, row: &Row) -> Result<Option<T>, DatabaseError>
    where
        T: DeserializeOwned,
    {
        serde_json::from_value(row.data.clone())
            .map(Some)
            .map_err(|e| DatabaseError::InvalidData(format!("Deserialization error: {}", e)))
    }

    fn execute_delete<T>(
        &self,
        table: &mut Table,
        key: &str,
        value: &str,
    ) -> Result<Option<T>, DatabaseError>
    where
        T: DeserializeOwned,
    {
        // Identify the `_id` of the row to be deleted.
        let target_id = table.rows.iter().find_map(|(id, row)| {
            if let Some(field_value) = row.data.get(key) {
                if field_value.as_str() == Some(value) {
                    return Some(id.clone());
                }
            }
            None
        });

        if let Some(target_id) = target_id {
            // Remove the row and deserialize the record.
            let row = table.rows.remove(&target_id).ok_or_else(|| {
                DatabaseError::InvalidData(
                    "Row unexpectedly not found during deletion.".to_string(),
                )
            })?;

            let record = serde_json::from_value(row.data).map_err(DatabaseError::JSONError)?;
            tracing::info!("Record deleted successfully.");
            return Ok(Some(record));
        }

        Ok(None) // No matching record found
    }

    pub async fn all<T>(&self) -> Vec<T>
    where
        T: DeserializeOwned,
    {
        let db = Database::load_from_file(&self.db_file_name)
            .await
            .unwrap_or_else(|e| {
                tracing::error!("Failed to load database from file: {}", e);
                Database {
                    name: String::new(),
                    file_name: self.db_file_name.clone(),
                    tables: HashMap::new(),
                }
            });
        self.handle_all(&db) // Shared logic
    }

    fn handle_all<T>(&self, db: &Database) -> Vec<T>
    where
        T: DeserializeOwned,
    {
        if let Some(table_name) = &self.table_name {
            if let Some(table) = db.tables.get(table_name) {
                let mut rows: Vec<&Row> = table.rows.values().collect();

                // Apply condition filters
                if !self.conditions.is_empty() {
                    rows.retain(|row| self.evaluate_conditions(&row.data));
                }

                // Apply sorting
                if !self.order_by.is_empty() {
                    rows.sort_by(|a, b| self.compare_rows(a, b));
                }

                // Apply offset
                let start = self.offset.unwrap_or(0);
                if start >= rows.len() {
                    return Vec::new();
                }

                // Apply limit
                let end = if let Some(limit) = self.limit {
                    std::cmp::min(start + limit, rows.len())
                } else {
                    rows.len()
                };

                // Convert to result type
                rows[start..end]
                    .iter()
                    .filter_map(|row| serde_json::from_value(row.data.clone()).ok())
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }

    // Helper: Evaluate all condition groups (they're combined with AND by default)
    fn evaluate_conditions(&self, row_data: &Value) -> bool {
        if self.conditions.is_empty() {
            return true;
        }

        // All condition groups must be true (implicitly ANDed together)
        self.conditions.iter().all(|group| group.evaluate(row_data))
    }

    // Helper: Compare two rows based on order_by clauses
    fn compare_rows(&self, a: &Row, b: &Row) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        for (field, sort_order) in &self.order_by {
            let a_val = a.data.get(field);
            let b_val = b.data.get(field);

            let ordering = match (a_val, b_val) {
                (Some(a_v), Some(b_v)) => self.compare_values(a_v, b_v),
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                (None, None) => Ordering::Equal,
            };

            if ordering != Ordering::Equal {
                return match sort_order {
                    crate::SortOrder::Asc => ordering,
                    crate::SortOrder::Desc => ordering.reverse(),
                };
            }
        }

        Ordering::Equal
    }

    // Helper: Compare two JSON values
    fn compare_values(&self, a: &Value, b: &Value) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match (a, b) {
            (Value::Number(a_num), Value::Number(b_num)) => {
                if let (Some(a_f), Some(b_f)) = (a_num.as_f64(), b_num.as_f64()) {
                    a_f.partial_cmp(&b_f).unwrap_or(Ordering::Equal)
                } else {
                    Ordering::Equal
                }
            }
            (Value::String(a_str), Value::String(b_str)) => a_str.cmp(b_str),
            (Value::Bool(a_bool), Value::Bool(b_bool)) => a_bool.cmp(b_bool),
            _ => Ordering::Equal,
        }
    }

    pub fn set(mut self, update_data: Value) -> Self {
        self.update_data = Some(update_data);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    use crate::setup_temp_db;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
    struct TestData {
        id: String,
        name: String,
    }

    #[test]
    fn test_query_from() {
        let query = Query {
            db_file_name: "test_db.json".into(),
            table_name: None,
            operation: Operation::Read,
            update_data: None,
            row_data: None,
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        };

        let updated_query = query.from("TestTable");
        assert_eq!(updated_query.table_name, Some("TestTable".to_string()));
    }

    #[test]
    fn test_query_data() {
        let query = Query {
            db_file_name: "test_db.json".into(),
            table_name: Some("TestTable".to_string()),
            operation: Operation::Update,
            update_data: None,
            row_data: None,
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        };

        let data = json!({ "name": "Updated Name" });
        let updated_query = query.data(data.clone());
        assert_eq!(updated_query.update_data, Some(data));
        std::fs::remove_file("test_db.json").ok();
    }

    #[test]
    fn test_query_data_from_struct() {
        std::fs::remove_file("test_db.json").ok();
        let query = Query {
            db_file_name: "test_db.json".into(),
            table_name: Some("TestTable".to_string()),
            operation: Operation::Create,
            update_data: None,
            row_data: None,
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        };

        let test_data = TestData {
            id: "123".to_string(),
            name: "John Doe".to_string(),
        };

        let updated_query = query.data_from_struct(test_data.clone());
        let expected_data = serde_json::to_value(test_data).unwrap();

        assert_eq!(updated_query.row_data, Some(expected_data));
    }

    #[tokio::test]
    async fn test_query_all() {
        let mut db = setup_temp_db().await;

        let test_data1 = TestData {
            id: "1".to_string(),
            name: "Alice".to_string(),
        };
        let test_data2 = TestData {
            id: "2".to_string(),
            name: "Bob".to_string(),
        };

        db.add_row()
            .from("TestTable")
            .data_from_struct(test_data1.clone())
            .execute_add()
            .await
            .expect("Failed to add row 1");

        db.add_row()
            .from("TestTable")
            .data_from_struct(test_data2.clone())
            .execute_add()
            .await
            .expect("Failed to add row 2");

        let rows: Vec<TestData> = db.get_rows().from("TestTable").all().await;

        assert_eq!(rows.len(), 2);
        assert!(rows.contains(&test_data1));
        assert!(rows.contains(&test_data2));

        // No explicit cleanup needed; tempfile will handle it automatically
    }

    #[test]
    fn test_query_set() {
        let query = Query {
            db_file_name: "test_db.json".into(),
            table_name: Some("TestTable".to_string()),
            operation: Operation::Update,
            update_data: None,
            row_data: None,
            conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        };

        let data = json!({ "name": "Updated Name" });
        let updated_query = query.set(data.clone());
        assert_eq!(updated_query.update_data, Some(data));
    }

    #[tokio::test]
    async fn test_query_where_eq_no_match() {
        let mut db = setup_temp_db().await;

        let test_data = TestData {
            id: "1".to_string(),
            name: "Alice".to_string(),
        };

        db.add_row()
            .from("TestTable")
            .data_from_struct(test_data)
            .execute_add()
            .await
            .expect("Failed to add row");

        let result: Option<TestData> = db
            .get_rows()
            .from("TestTable")
            .where_eq("id", "999")
            .await
            .unwrap();
        assert!(result.is_none(), "Expected no matching record");
    }

    #[tokio::test]
    async fn test_query_where_eq_match() {
        let mut db = setup_temp_db().await;

        let test_data = TestData {
            id: "1".to_string(),
            name: "Alice".to_string(),
        };

        db.add_row()
            .from("TestTable")
            .data_from_struct(test_data.clone())
            .execute_add()
            .await
            .expect("Failed to add row");

        let result: Option<TestData> = db
            .get_rows()
            .from("TestTable")
            .where_eq("id", "1")
            .await
            .unwrap();
        assert_eq!(result, Some(test_data), "Expected matching record");
    }

    #[tokio::test]
    async fn test_query_execute_update() {
        let mut db = setup_temp_db().await;

        let test_data = TestData {
            id: "1".to_string(),
            name: "Alice".to_string(),
        };

        db.add_row()
            .from("TestTable")
            .data_from_struct(test_data.clone())
            .execute_add()
            .await
            .expect("Failed to add row");

        let update_data = json!({ "name": "Updated Alice" });

        let result = db
            .update_row()
            .from("TestTable")
            .data(update_data)
            .where_eq::<TestData>("id", "1")
            .await
            .unwrap();

        assert!(
            result.is_some(),
            "Expected update to return the updated record"
        );
        assert_eq!(
            result.unwrap().name,
            "Updated Alice",
            "Name was not updated"
        );
    }

    #[tokio::test]
    async fn test_query_execute_delete() {
        let mut db = setup_temp_db().await;

        let test_data = TestData {
            id: "1".to_string(),
            name: "Alice".to_string(),
        };

        db.add_row()
            .from("TestTable")
            .data_from_struct(test_data.clone())
            .execute_add()
            .await
            .expect("Failed to add row");

        let result = db
            .get_single()
            .from("TestTable")
            .where_eq::<TestData>("id", "1")
            .await
            .unwrap();

        assert!(result.is_some(), "Expected record to exist before deletion");

        let _ = db
            .delete_single()
            .from("TestTable")
            .where_eq::<TestData>("id", "1")
            .await
            .unwrap();

        let rows: Vec<TestData> = db.get_rows().from("TestTable").all().await;
        let deleted_record = db
            .get_single()
            .from("TestTable")
            .where_eq::<TestData>("id", "1")
            .await
            .unwrap();

        assert!(deleted_record.is_none(), "Expected record to be deleted");
        assert!(rows.is_empty(), "Expected all records to be deleted");
    }

    // New tests for enhanced query operations
    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
    struct Person {
        id: String,
        name: String,
        age: u32,
        status: String,
    }

    async fn setup_person_db() -> Database {
        use crate::{Columns, Table};
        let mut db = setup_temp_db().await;
        
        // Create a proper table for Person struct
        let person_columns = Columns::from_struct::<Person>(true);
        let mut person_table = Table::new("PersonTable".to_string(), person_columns);
        db.add_table(&mut person_table).await.expect("Failed to create PersonTable");
        
        let people = vec![
            Person { id: "1".into(), name: "Alice".into(), age: 25, status: "active".into() },
            Person { id: "2".into(), name: "Bob".into(), age: 35, status: "active".into() },
            Person { id: "3".into(), name: "Charlie".into(), age: 45, status: "inactive".into() },
            Person { id: "4".into(), name: "Diana".into(), age: 30, status: "active".into() },
            Person { id: "5".into(), name: "Eve".into(), age: 28, status: "inactive".into() },
        ];

        for person in people {
            db.add_row()
                .from("PersonTable")
                .data_from_struct(person)
                .execute_add()
                .await
                .expect("Failed to add person");
        }

        db
    }

    #[tokio::test]
    async fn test_where_gt() {
        let db = setup_person_db().await;

        let people: Vec<Person> = db
            .get_rows()
            .from("PersonTable")
            .where_gt("age", 30)
            .all()
            .await;

        assert_eq!(people.len(), 2, "Should find 2 people over 30");
        assert!(people.iter().all(|p| p.age > 30));
    }

    #[tokio::test]
    async fn test_where_lt() {
        let db = setup_person_db().await;

        let people: Vec<Person> = db
            .get_rows()
            .from("PersonTable")
            .where_lt("age", 30)
            .all()
            .await;

        assert_eq!(people.len(), 2, "Should find 2 people under 30");
        assert!(people.iter().all(|p| p.age < 30));
    }

    #[tokio::test]
    async fn test_where_gte() {
        let db = setup_person_db().await;

        let people: Vec<Person> = db
            .get_rows()
            .from("PersonTable")
            .where_gte("age", 30)
            .all()
            .await;

        assert_eq!(people.len(), 3, "Should find 3 people 30 or older");
        assert!(people.iter().all(|p| p.age >= 30));
    }

    #[tokio::test]
    async fn test_where_lte() {
        let db = setup_person_db().await;

        let people: Vec<Person> = db
            .get_rows()
            .from("PersonTable")
            .where_lte("age", 30)
            .all()
            .await;

        assert_eq!(people.len(), 3, "Should find 3 people 30 or younger");
        assert!(people.iter().all(|p| p.age <= 30));
    }

    #[tokio::test]
    async fn test_order_by_asc() {
        let db = setup_person_db().await;

        let people: Vec<Person> = db
            .get_rows()
            .from("PersonTable")
            .order_by("name", crate::SortOrder::Asc)
            .all()
            .await;

        assert_eq!(people.len(), 5);
        assert_eq!(people[0].name, "Alice");
        assert_eq!(people[1].name, "Bob");
        assert_eq!(people[2].name, "Charlie");
        assert_eq!(people[3].name, "Diana");
        assert_eq!(people[4].name, "Eve");
    }

    #[tokio::test]
    async fn test_order_by_desc() {
        let db = setup_person_db().await;

        let people: Vec<Person> = db
            .get_rows()
            .from("PersonTable")
            .order_by("age", crate::SortOrder::Desc)
            .all()
            .await;

        assert_eq!(people.len(), 5);
        assert_eq!(people[0].age, 45);
        assert_eq!(people[1].age, 35);
        assert_eq!(people[2].age, 30);
    }

    #[tokio::test]
    async fn test_limit() {
        let db = setup_person_db().await;

        let people: Vec<Person> = db
            .get_rows()
            .from("PersonTable")
            .limit(3)
            .all()
            .await;

        assert_eq!(people.len(), 3, "Should return only 3 results");
    }

    #[tokio::test]
    async fn test_offset() {
        let db = setup_person_db().await;

        let people: Vec<Person> = db
            .get_rows()
            .from("PersonTable")
            .order_by("age", crate::SortOrder::Asc)
            .offset(2)
            .all()
            .await;

        assert_eq!(people.len(), 3, "Should skip first 2 results");
        assert_eq!(people[0].age, 30);
    }

    #[tokio::test]
    async fn test_limit_and_offset() {
        let db = setup_person_db().await;

        let people: Vec<Person> = db
            .get_rows()
            .from("PersonTable")
            .order_by("age", crate::SortOrder::Asc)
            .offset(1)
            .limit(2)
            .all()
            .await;

        assert_eq!(people.len(), 2, "Should return 2 results starting from offset 1");
        assert_eq!(people[0].age, 28);
        assert_eq!(people[1].age, 30);
    }

    #[tokio::test]
    async fn test_complex_query_with_conditions() {
        let db = setup_person_db().await;

        // Find active people over 25, ordered by age, limit to 2
        let people: Vec<Person> = db
            .get_rows()
            .from("PersonTable")
            .where_gt("age", 25)
            .order_by("age", crate::SortOrder::Asc)
            .limit(2)
            .all()
            .await;

        assert_eq!(people.len(), 2);
        assert_eq!(people[0].age, 28);
        assert_eq!(people[1].age, 30);
    }

    #[tokio::test]
    async fn test_multiple_conditions_same_group() {
        let db = setup_person_db().await;

        // Should combine with AND - age > 25 AND age < 40
        let people: Vec<Person> = db
            .get_rows()
            .from("PersonTable")
            .where_gt("age", 25)
            .where_lt("age", 40)
            .all()
            .await;

        assert_eq!(people.len(), 3, "Should find 3 people between 25 and 40");
        assert!(people.iter().all(|p| p.age > 25 && p.age < 40));
    }

    #[tokio::test]
    async fn test_multi_column_sort() {
        use crate::{Columns, Table};
        let mut db = setup_temp_db().await;
        
        // Create PersonTable for this test
        let person_columns = Columns::from_struct::<Person>(true);
        let mut person_table = Table::new("PersonTable".to_string(), person_columns);
        db.add_table(&mut person_table).await.expect("Failed to create PersonTable");

        let people = vec![
            Person { id: "1".into(), name: "Alice".into(), age: 30, status: "active".into() },
            Person { id: "2".into(), name: "Bob".into(), age: 30, status: "active".into() },
            Person { id: "3".into(), name: "Charlie".into(), age: 25, status: "active".into() },
        ];

        for person in people {
            db.add_row()
                .from("PersonTable")
                .data_from_struct(person)
                .execute_add()
                .await
                .expect("Failed to add person");
        }

        // Sort by age DESC, then by name ASC
        let results: Vec<Person> = db
            .get_rows()
            .from("PersonTable")
            .order_by("age", crate::SortOrder::Desc)
            .order_by("name", crate::SortOrder::Asc)
            .all()
            .await;

        assert_eq!(results.len(), 3);
        // Age 30 comes first (DESC), and within age 30, Alice before Bob (ASC)
        assert_eq!(results[0].name, "Alice");
        assert_eq!(results[1].name, "Bob");
        assert_eq!(results[2].name, "Charlie");
    }

    #[test]
    fn test_display_comparison_op() {
        use crate::ComparisonOp;
        assert_eq!(format!("{}", ComparisonOp::Eq), "=");
        assert_eq!(format!("{}", ComparisonOp::Ne), "!=");
        assert_eq!(format!("{}", ComparisonOp::Gt), ">");
        assert_eq!(format!("{}", ComparisonOp::Gte), ">=");
        assert_eq!(format!("{}", ComparisonOp::Lt), "<");
        assert_eq!(format!("{}", ComparisonOp::Lte), "<=");
        assert_eq!(format!("{}", ComparisonOp::Like), "LIKE");
        assert_eq!(format!("{}", ComparisonOp::In), "IN");
        assert_eq!(format!("{}", ComparisonOp::NotIn), "NOT IN");
    }

    #[test]
    fn test_display_logical_op() {
        use crate::LogicalOp;
        assert_eq!(format!("{}", LogicalOp::And), "AND");
        assert_eq!(format!("{}", LogicalOp::Or), "OR");
    }

    #[test]
    fn test_display_sort_order() {
        use crate::SortOrder;
        assert_eq!(format!("{}", SortOrder::Asc), "ASC");
        assert_eq!(format!("{}", SortOrder::Desc), "DESC");
    }

    #[test]
    fn test_display_condition() {
        use crate::{Condition, ComparisonOp};
        use serde_json::json;

        // Simple equality
        let cond = Condition::new("age", ComparisonOp::Eq, json!(30));
        assert_eq!(format!("{}", cond), "age = 30");

        // String value
        let cond = Condition::new("name", ComparisonOp::Like, json!("John%"));
        assert_eq!(format!("{}", cond), "name LIKE 'John%'");

        // Greater than
        let cond = Condition::new("score", ComparisonOp::Gt, json!(100));
        assert_eq!(format!("{}", cond), "score > 100");

        // IN with array
        let cond = Condition::new("status", ComparisonOp::In, json!(["active", "pending"]));
        assert_eq!(format!("{}", cond), "status IN ('active', 'pending')");

        // NOT IN with numbers
        let cond = Condition::new("id", ComparisonOp::NotIn, json!([1, 2, 3]));
        assert_eq!(format!("{}", cond), "id NOT IN (1, 2, 3)");
    }

    #[tokio::test]
    async fn test_display_database() {
        use crate::util::setup_temp_db;

        let mut db = setup_temp_db().await;
        
        // setup_temp_db creates a TestTable, so we have 1 table
        let display = format!("{}", db);
        assert!(display.contains("Database"));
        assert!(display.contains("1 tables"));
        assert!(display.contains("TestTable"));

        // Add another table
        let columns = crate::Columns::from_struct::<Person>(true);
        let mut table = crate::Table::new("users".to_string(), columns);
        db.add_table(&mut table).await.unwrap();

        let display = format!("{}", db);
        assert!(display.contains("Database"));
        assert!(display.contains("2 tables"));
        assert!(display.contains("users"));
    }

    #[tokio::test]
    async fn test_display_table() {
        use crate::util::setup_temp_db;

        let mut db = setup_temp_db().await;
        let columns = crate::Columns::from_struct::<Person>(true);
        let mut table = crate::Table::new("people".to_string(), columns.clone());

        // Empty table
        let display = format!("{}", table);
        assert!(display.contains("Table 'people'"));
        assert!(display.contains("0 rows"));
        assert!(display.contains(&format!("{} columns", columns.0.len())));

        // Add table to db and add rows
        db.add_table(&mut table).await.unwrap();
        
        db.add_row()
            .from("people")
            .data_from_struct(Person {
                id: "1".to_string(),
                name: "Alice".to_string(),
                age: 30,
                status: "active".to_string(),
            })
            .execute_add()
            .await
            .unwrap();

        // Get updated table reference
        db.reload().await.unwrap();
        let display = format!("{}", db.tables.get("people").unwrap());
        assert!(display.contains("Table 'people'"));
        assert!(display.contains("1 rows"));
    }

    #[tokio::test]
    async fn test_display_query() {
        use crate::util::setup_temp_db;

        let db = setup_temp_db().await;

        // Simple query
        let query = db.get_rows().from("users");
        let display = format!("{}", query);
        assert!(display.contains("Query["));
        assert!(display.contains("table: users"));
        assert!(display.contains("op: Read"));

        // Query with conditions
        let query = db
            .get_rows()
            .from("users")
            .where_gt("age", 25)
            .where_lt("age", 65);
        let display = format!("{}", query);
        assert!(display.contains("where:"));
        assert!(display.contains("age > 25"));
        assert!(display.contains("age < 65"));

        // Query with ordering
        let query = db
            .get_rows()
            .from("users")
            .order_by("name", crate::SortOrder::Asc)
            .order_by("age", crate::SortOrder::Desc);
        let display = format!("{}", query);
        assert!(display.contains("order:"));
        assert!(display.contains("name ASC"));
        assert!(display.contains("age DESC"));

        // Query with limit and offset
        let query = db
            .get_rows()
            .from("users")
            .limit(10)
            .offset(20);
        let display = format!("{}", query);
        assert!(display.contains("limit: 10"));
        assert!(display.contains("offset: 20"));

        // Complex query
        let query = db
            .get_rows()
            .from("orders")
            .where_gte("total", 100)
            .where_ne("status", "cancelled")
            .order_by("created_at", crate::SortOrder::Desc)
            .limit(50);
        let display = format!("{}", query);
        assert!(display.contains("table: orders"));
        assert!(display.contains("total >= 100"));
        assert!(display.contains("status != 'cancelled'"));
        assert!(display.contains("created_at DESC"));
        assert!(display.contains("limit: 50"));
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Query[")?;
        
        // Table name
        if let Some(ref table) = self.table_name {
            write!(f, "table: {}", table)?;
        } else {
            write!(f, "table: <none>")?;
        }
        
        // Operation type
        write!(f, ", op: {:?}", self.operation)?;
        
        // Conditions
        if !self.conditions.is_empty() {
            write!(f, ", where: ")?;
            for (i, group) in self.conditions.iter().enumerate() {
                if i > 0 {
                    write!(f, " {} ", group.operator)?;
                }
                write!(f, "(")?;
                for (j, cond) in group.conditions.iter().enumerate() {
                    if j > 0 {
                        write!(f, " {} ", group.operator)?;
                    }
                    write!(f, "{}", cond)?;
                }
                write!(f, ")")?;
            }
        }
        
        // Order by
        if !self.order_by.is_empty() {
            write!(f, ", order: ")?;
            for (i, (field, order)) in self.order_by.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{} {}", field, order)?;
            }
        }
        
        // Limit
        if let Some(limit) = self.limit {
            write!(f, ", limit: {}", limit)?;
        }
        
        // Offset
        if let Some(offset) = self.offset {
            write!(f, ", offset: {}", offset)?;
        }
        
        write!(f, "]")
    }
}

#[cfg(test)]
mod bulk_tests {
    use super::*;
    use crate::util::setup_temp_db;
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
    struct TestUser {
        id: String,
        name: String,
        age: u32,
        status: String,
    }

    async fn setup_users_db() -> Database {
        use crate::{Columns, Table};
        let mut db = setup_temp_db().await;
        
        // Create users table
        let user_columns = Columns::from_struct::<TestUser>(true);
        let mut users_table = Table::new("users".to_string(), user_columns);
        db.add_table(&mut users_table).await.expect("Failed to create users table");
        
        db
    }

    #[tokio::test]
    async fn test_bulk_insert_many_users() {
        let mut db = setup_users_db().await;

        let users = vec![
            TestUser {
                id: "1".to_string(),
                name: "Alice".to_string(),
                age: 30,
                status: "active".to_string(),
            },
            TestUser {
                id: "2".to_string(),
                name: "Bob".to_string(),
                age: 25,
                status: "active".to_string(),
            },
            TestUser {
                id: "3".to_string(),
                name: "Charlie".to_string(),
                age: 35,
                status: "inactive".to_string(),
            },
        ];

        let inserted = db
            .insert_many()
            .from("users")
            .data_many(users)
            .execute_bulk_insert()
            .await
            .expect("Bulk insert failed");

        assert_eq!(inserted, 3);
        
        // Reload to sync state
        db.reload().await.unwrap();
        assert_eq!(db.count_rows("users").unwrap(), 3);
    }

    #[tokio::test]
    async fn test_bulk_insert_with_json_array() {
        let mut db = setup_users_db().await;

        let users_json = json!([
            {"id": "1", "name": "Alice", "age": 30, "status": "active"},
            {"id": "2", "name": "Bob", "age": 25, "status": "active"},
        ]);

        let inserted = db
            .insert_many()
            .from("users")
            .data(users_json)
            .execute_bulk_insert()
            .await
            .expect("Bulk insert failed");

        assert_eq!(inserted, 2);
        
        // Verify
        db.reload().await.unwrap();
        assert_eq!(db.count_rows("users").unwrap(), 2);
    }

    #[tokio::test]
    async fn test_bulk_update_matching_condition() {
        let mut db = setup_users_db().await;

        // Insert test data
        let users = vec![
            TestUser { id: "1".to_string(), name: "Alice".to_string(), age: 30, status: "active".to_string() },
            TestUser { id: "2".to_string(), name: "Bob".to_string(), age: 25, status: "active".to_string() },
            TestUser { id: "3".to_string(), name: "Charlie".to_string(), age: 35, status: "active".to_string() },
        ];
        db.insert_many().from("users").data_many(users).execute_bulk_insert().await.unwrap();

        // Update all users with age > 28
        let updated = db
            .update_many()
            .from("users")
            .data(json!({"status": "senior"}))
            .where_gt("age", 28)
            .execute_bulk_update()
            .await
            .expect("Bulk update failed");

        assert_eq!(updated, 2); // Alice (30) and Charlie (35)

        // Verify updates
        db.reload().await.unwrap();
        let table = db.tables.get("users").unwrap();
        let senior_count = table.rows.values()
            .filter(|row| row.data.get("status").and_then(|v| v.as_str()) == Some("senior"))
            .count();
        
        assert_eq!(senior_count, 2);
    }

    #[tokio::test]
    async fn test_bulk_update_all_rows() {
        let mut db = setup_users_db().await;

        // Insert test data
        let users = vec![
            TestUser { id: "1".to_string(), name: "Alice".to_string(), age: 30, status: "active".to_string() },
            TestUser { id: "2".to_string(), name: "Bob".to_string(), age: 25, status: "inactive".to_string() },
        ];
        db.insert_many().from("users").data_many(users).execute_bulk_insert().await.unwrap();

        // Update all users (no conditions)
        let updated = db
            .update_many()
            .from("users")
            .data(json!({"status": "verified"}))
            .execute_bulk_update()
            .await
            .expect("Bulk update failed");

        assert_eq!(updated, 2);
    }

    #[tokio::test]
    async fn test_bulk_delete_matching_condition() {
        let mut db = setup_users_db().await;

        // Insert test data
        let users = vec![
            TestUser { id: "1".to_string(), name: "Alice".to_string(), age: 30, status: "active".to_string() },
            TestUser { id: "2".to_string(), name: "Bob".to_string(), age: 25, status: "inactive".to_string() },
            TestUser { id: "3".to_string(), name: "Charlie".to_string(), age: 35, status: "inactive".to_string() },
        ];
        db.insert_many().from("users").data_many(users).execute_bulk_insert().await.unwrap();

        // Delete all inactive users
        let deleted = db
            .delete_many()
            .from("users")
            .where_equals("status", "inactive")
            .execute_bulk_delete()
            .await
            .expect("Bulk delete failed");

        assert_eq!(deleted, 2); // Bob and Charlie

        // Verify only Alice remains
        db.reload().await.unwrap();
        assert_eq!(db.count_rows("users").unwrap(), 1);
    }

    #[tokio::test]
    async fn test_bulk_delete_with_multiple_conditions() {
        let mut db = setup_users_db().await;

        // Insert test data
        let users = vec![
            TestUser { id: "1".to_string(), name: "Alice".to_string(), age: 30, status: "active".to_string() },
            TestUser { id: "2".to_string(), name: "Bob".to_string(), age: 25, status: "active".to_string() },
            TestUser { id: "3".to_string(), name: "Charlie".to_string(), age: 35, status: "active".to_string() },
        ];
        db.insert_many().from("users").data_many(users).execute_bulk_insert().await.unwrap();

        // Delete users where age < 28 AND status = "active"
        let deleted = db
            .delete_many()
            .from("users")
            .where_lt("age", 28)
            .where_equals("status", "active")
            .execute_bulk_delete()
            .await
            .expect("Bulk delete failed");

        assert_eq!(deleted, 1); // Only Bob (age 25)

        // Verify Alice and Charlie remain
        db.reload().await.unwrap();
        assert_eq!(db.count_rows("users").unwrap(), 2);
    }

    #[tokio::test]
    async fn test_bulk_operations_performance() {
        let mut db = setup_users_db().await;

        // Generate 100 users
        let users: Vec<TestUser> = (1..=100)
            .map(|i| TestUser {
                id: i.to_string(),
                name: format!("User{}", i),
                age: 20 + (i % 50),
                status: if i % 2 == 0 { "active" } else { "inactive" }.to_string(),
            })
            .collect();

        // Bulk insert should be fast
        let start = std::time::Instant::now();
        let inserted = db
            .insert_many()
            .from("users")
            .data_many(users)
            .execute_bulk_insert()
            .await
            .expect("Bulk insert failed");
        let duration = start.elapsed();

        assert_eq!(inserted, 100);
        assert!(duration.as_millis() < 100, "Bulk insert took too long: {:?}", duration);

        // Bulk update
        let updated = db
            .update_many()
            .from("users")
            .where_equals("status", "active")
            .data(json!({"status": "verified"}))
            .execute_bulk_update()
            .await
            .expect("Bulk update failed");

        assert_eq!(updated, 50); // 50 even-numbered users
    }
}
