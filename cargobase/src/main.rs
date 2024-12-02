// use cargobase::init_tracing;
// use cargobase::{Column, Columns, Database, Table};
use cargobase::Database;

use serde::{Deserialize, Serialize};
// use serde_json::json;
use std::error::Error;
// use uuid::Uuid;
use tracing_subscriber::fmt;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TestData {
    pub id: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
}
// #[derive(Debug, Serialize, Deserialize, Default)]
// pub struct Testing {
//     pub id: String,
//     pub location: String,
//     pub age: String,
// }
// #[derive(Debug, Serialize, Deserialize, Default)]
// pub struct ThisErrorTesting {
//     pub id: String,
//     pub location: String,
//     pub age: String,
// }

fn main() -> Result<(), Box<dyn Error>> {
    init_tracing();
    let db = Database::new("cargobase");
    // let mut db = Database::new("cargobase");
    // db.drop_database()?;
    // let mut db = Database::new("cargobase");
    // let mut db = Database::new("cargobase_rows");
    // println!("{:#?}", db);
    //
    //
    //
    //
    //
    //
    //
    // let mut db = Database::new("TestUpdateAndDelete");
    // let mut test_table = Table::new(
    //     "TestTracingInfo".to_string(),
    //     Columns::from_struct::<TestData>(true),
    // );
    // println!("{:#?}", test_table);
    // db.add_table(&mut test_table)?;
    // let record1 = TestData {
    //     id: Some(Uuid::new_v4().to_string()),
    //     name: Some("Jon Doe".to_string()),
    //     email: Some("jondoe@email.com".to_string()),
    // };
    // let record2 = TestData {
    //     id: Some(Uuid::new_v4().to_string()),
    //     name: Some("Jane Doe".to_string()),
    //     email: Some("janedoe@email.com".to_string()),
    // };
    // let record3 = TestData {
    //     id: Some(Uuid::new_v4().to_string()),
    //     name: Some("alice cooper".to_string()),
    //     email: None,
    // };
    // let test_data = vec![record1, record2, record3];
    // test_data.iter().for_each(|data| {
    //     db.add_row()
    //         .from("TestTracingInfo")
    //         .data_from_struct::<TestData>(data.clone())
    //         .execute_add()
    //         .expect("Failed to add row");
    // });
    //
    //
    //
    // db.update_row()
    //     .from("TestTracingInfo")
    //     .data(json!({
    //         "id": "4",
    //         "name": "Abuja",
    //         "email": "99",
    //     }))
    //     .where_eq::<TestData>("id", "0727f3ca-89ca-4d4d-88cd-598d2979645b")?;
    //
    //
    // db.delete_single()
    //     .from("TestTracingInfo")
    //     .where_eq::<TestData>("id", "4")?;
    //
    //
    //
    //
    //
    //
    //
    //
    //
    // let mut test_table = Table::new("TestNoFileName".to_string(), Columns::from_struct::<TestData>(true));

    // db.drop_table("TestNoFileName")?;
    //

    // db.add_row()
    //     .from("Test")
    //     .data_from_struct::<TestData>(record1)
    //     .execute_add()
    //     .expect("Failed to add row");
    //
    // db.add_row()
    //     .from("Test")
    //     .data_from_struct::<TestData>(record2)
    //     .execute_add()
    //     .expect("Failed to add row");

    // let all_test_data: Vec<TestData> = db.get_rows().from("Test").all();
    //
    // all_test_data.iter().for_each(|data| {
    //     println!("{:#?}", data.email);
    // });
    // let rows: Vec<TestData> = db.get_rows().from("Test").all();
    // println!("rows! {:#?}", rows);

    // let mut this_error_table_ordered = Table::new("ThisErrorTestingOrdered".to_string(), Columns::from_struct::<ThisErrorTesting>(true));

    // let _ = db.add_table(&mut this_error_table_ordered);
    // let _ = db.drop_table("TestTracingInfo");

    db.view();
    // db.view_table("Test_from_struct");
    Ok(())
}

// #[cfg(feature = "logging")]
pub fn init_tracing() {
    let subscriber = fmt::Subscriber::builder()
        .with_max_level(tracing::Level::WARN)
        .finish();
    /*
    example implementation:
    info!(target: "cargobase", "Database `{name}` already exists, loading...");
    */
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}
// ************************************************************************** //
// ************************************************************************** //
// ************************************************************************** //
// ************************************************************************** //
// ************************************************************************** //
// ************************************************************************** //
//
//
//
// let new_row = Testing {
//     id: "3333".to_string(),
//     location: "^^^^over there".to_string(),
//     age: "?????fml rust is hard".to_string(),
// };
//
// db.add_row()
//     .to("Test_from_struct")
//     .data_from_struct::<Testing>(new_row)
//     .execute_add()
//     .expect("Failed to add row");
//
//
// #[derive(Debug, Serialize, Deserialize)]
// pub struct User {
//     pub id: String,
//     pub name: String,
//     pub age: String,
// }

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

// let test_colums = Columns::new(vec![
//     Column::new("id", true),
//     Column::new("name", true),
//     Column::new("email", true),
// ]);

// let mut users_table = Table::new("Users".to_string(), user_columns);
// let mut posts_table = Table::new("Posts".to_string(), post_columns);
// let mut test_table = Table::new("Test".to_string(), test_colums);

//
// db.add_table(&mut users_table);
// db.add_table(&mut posts_table);
// db.add_table(&mut test_table);
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

// let test_data = vec![
//     json!({
//         "id": Uuid::new_v4().to_string(),
//         "name": "Jon Doe",
//         "email": "jondoe@email.com"
//     }),
//     json!({
//         "id": Uuid::new_v4().to_string(),
//         "name": "Jane Doe",
//         "email": "janedoe@email.com"
//     }),
// ];
//
// users_table.add_row(&mut db, json!(users));
// posts_table.add_row(&mut db, json!(posts));
// test_table.add_row(&mut db, json!(test_data));

// let all_test_data: Vec<TestData> = db.get_rows().from("Test").all();

// let all_rows: Vec<User> = db.get_rows().from("Users").all();
// println!("{:#?}", all_rows.len());
// println!("{:#?}", all_rows);
// all_rows.iter().for_each(|user| {
//     println!("{:#?}", user.id);
// });

// println!("db: {:#?}", db);

// let single_user: User = db
//     .get_single()
//     .from("Users")
//     .where_eq("id", "39fe81e3-23a3-4212-b608-c4d11b33d995")?;
//
// println!("{:#?}", single_user);

// let single_user_to_delete: User = db
//     .delete_single()
//     .from("Users")
//     .where_eq("id", "597c4e6d-1a3b-4618-a0f5-7bfb71a243f0")?;

// println!("{:#?}", single_user_to_delete);

// "597c4e6d-1a3b-4618-a0f5-7bfb71a243f0"

// let all_rows: Vec<User> = db.get_rows().from("Users").all();
// println!("{:#?}", all_rows.len());
// let _ = db.drop_table("Droppable");
//
//
//
//
//
//
// let test_columns = vec![
//     Column::new("id", true),
//     Column::new("name", true),
//     Column::new("email", true),
// ];

// let mut test_table = Table::new("Test".to_string(), Columns::new(test_columns));

// let test_columns = Columns::new(vec![
//     Column::new("id", true),
//     Column::new("location", true),
//     Column::new("age", true),
// ]);
//
// let test_columns_default = Columns::from_struct::<Testing>(true);
//
// let mut test_table_from_struct =
//     Table::new("Test_from_struct".to_string(), test_columns_default);
//
// println!("rows: {:?}", test_table_from_struct.rows);
// println!("columns: {:?}", test_table_from_struct.columns);

// TODO: update add_table method so it doesnt crash the program is the table already exists
//
// db.add_table(&mut test_table_from_struct)?;

// let testing_data = vec![
//     Testing {
//         id: "1".to_string(),
//         location: "Lagos".to_string(),
//         age: "30".to_string(),
//     },
//     Testing {
//         id: "2".to_string(),
//         location: "Abuja".to_string(),
//         age: "25".to_string(),
//     },
// ];
// //
// test_table_from_struct.add_row(&mut db, json!(testing_data));

// db.add_table(&mut test_table)?;

// db.tables.iter().for_each(|table| {
//     println!("{:#?}", table.name);
// });
//
//
//
// db.get_rows()
//     .from("Test")
//     .all()
//     .iter()
//     .for_each(|data: &TestData| {
//         println!("{:#?}", data.email);
//     });
//
// let record: Testing = db // must have the type/struct
//     .get_single()
//     .from("Test_from_struct")
//     .where_eq("id", "1")?;
// println!("record: {:#?}", record);

// let deleted_record = db
//     .delete_single()
//     .from("Test_from_struct")
//     .where_eq::<Testing>("id", "1")?;
//
// println!("deleted_record: {:#?}", deleted_record);

// db.update_row()
//     .from("Test_from_struct")
//     .data(json!({
//         "id": "4",
//         "location": "Abuja",
//         "age": "99",
//     }))
//     .where_eq::<Testing>("id", "2")?;
