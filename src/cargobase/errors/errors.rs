use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Failed to load the batabase: {0}")]
    LoadError(std::io::Error),

    #[error("Failed to save the database: {0}")]
    SaveError(std::io::Error),

    #[error("Failed to drop database: {0}")]
    // return Err(DatabaseError::DeleteError);
    DeleteError(String),

    #[error("Table `{0}` already exists")] // skipping creation
    TableAlreadyExists(String),

    #[error("Table {0} not found")]
    TableNotFound(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Row not found with {0} = {1}")]
    RowNotFound(String, String),

    #[error("Column `{0}` is missing from the row data")]
    MissingColumn(String),
}
