pub mod database;
pub mod query;
pub mod table;
pub mod util;
pub mod view;

pub mod columns;
pub mod errors;
pub mod row;

pub use columns::{Column, Columns};
pub use errors::DatabaseError;
pub use row::Row;

pub use database::Database;
pub use query::Query;
pub use table::Table;
pub use util::setup_temp_db;
pub use view::View;

use tracing_subscriber;
pub fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}
