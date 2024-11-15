mod database;

use database::database::{Column, Columns, Database, Table};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub age: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = Database::new("cargobase".to_string());

    // let user_columns = Columns::new(vec![
    //     Column::new("id", true),
    //     Column::new("name", true),
    //     Column::new("age", true),
    // ]);
    //
    // let post_columns = Columns::new(vec![
    //     Column::new("id", true),
    //     Column::new("title", true),
    //     Column::new("content", true),
    // ]);
    //
    // let mut users_table = Table::new("Users".to_string(), user_columns);
    // let mut posts_table = Table::new("Posts".to_string(), post_columns);
    //
    // db.add_table(&mut users_table);
    // db.add_table(&mut posts_table);
    //
    // let user1 = User {
    //     id: Uuid::new_v4().to_string(),
    //     name: "Jon Doe".to_string(),
    //     age: "30".to_string(),
    // };
    //
    // let user2 = User {
    //     id: Uuid::new_v4().to_string(),
    //     name: "Jane Doe".to_string(),
    //     age: "25".to_string(),
    // };
    //
    // let posts = vec![
    //     json!({
    //         "id": Uuid::new_v4().to_string(),
    //         "title": "Hello, world!",
    //         "content": "This is my first post.",
    //     }),
    //     json!({
    //         "id": Uuid::new_v4().to_string(),
    //         "title": "Hello, again!",
    //         "content": "This is my second post.",
    //     }),
    // ];
    //
    // let users = vec![user1, user2];
    //
    // users_table.add_row(&mut db, json!(users));
    // posts_table.add_row(&mut db, json!(posts));

    // let all_rows = users_table.get_rows_all();

    // let all_rows: Vec<User> = db.get_rows().from("Users").all();
    // println!("{:#?}", all_rows);
    // all_rows.iter().for_each(|user| {
    //     println!("{:#?}", user.id);
    // });

    // println!("db: {:#?}", db);

    let single_user: User = db
        .get_single()
        .from("Users")
        .where_eq("id", "39fe81e3-23a3-4212-b608-c4d11b33d995")?;

    println!("{:#?}", single_user);

    Ok(())
}
