use serde::{Deserialize, Serialize};

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
pub struct Columns(pub Vec<Column>);

impl Columns {
    pub fn new(columns: Vec<Column>) -> Self {
        Columns(columns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_new() {
        let column = Column::new("name", true);
        assert_eq!(column.name, "name");
        assert_eq!(column.required, true);
    }

    #[test]
    fn test_columns_new() {
        let columns = Columns::new(vec![Column::new("name", true), Column::new("age", false)]);
        assert_eq!(columns.0.len(), 2);
        assert_eq!(columns.0[0].name, "name");
        assert_eq!(columns.0[0].required, true);
        assert_eq!(columns.0[1].name, "age");
        assert_eq!(columns.0[1].required, false);
    }
}
