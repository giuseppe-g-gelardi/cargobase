use cargobase::{Column, Columns, Database, Table};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
struct User {
    id: String,
    name: String,
    age: u32,
    email: String,
    status: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 CargoBase Example - JSON Database Demo\n");

    // Create a new database
    let mut db = Database::new(Cow::Borrowed("example_db")).await;
    println!("✅ Created database: {}", db.name);
    println!("   Display: {}\n", db);

    // Define schema with columns
    let user_columns = Columns::new(vec![
        Column::new("id", true),
        Column::new("name", true),
        Column::new("age", true),
        Column::new("email", true),
        Column::new("status", false),
    ]);

    // Create a table
    let mut users_table = Table::new("users".to_string(), user_columns);
    db.add_table(&mut users_table).await?;
    println!("✅ Created 'users' table\n");

    // Bulk insert sample users (faster than individual inserts)
    println!("📝 Bulk inserting sample users...");
    let users = vec![
        User {
            id: "1".to_string(),
            name: "Alice Johnson".to_string(),
            age: 28,
            email: "alice@example.com".to_string(),
            status: "active".to_string(),
        },
        User {
            id: "2".to_string(),
            name: "Bob Smith".to_string(),
            age: 35,
            email: "bob@example.com".to_string(),
            status: "active".to_string(),
        },
        User {
            id: "3".to_string(),
            name: "Charlie Brown".to_string(),
            age: 42,
            email: "charlie@example.com".to_string(),
            status: "inactive".to_string(),
        },
        User {
            id: "4".to_string(),
            name: "Diana Prince".to_string(),
            age: 25,
            email: "diana@example.com".to_string(),
            status: "active".to_string(),
        },
    ];

    let inserted_count = db
        .insert_many()
        .from("users")
        .data_many(users.clone())
        .execute_bulk_insert()
        .await?;
    println!("  ✅ Bulk inserted {} users\n", inserted_count);

    // Reload to sync in-memory state
    db.reload().await?;
    println!("   Database after inserts: {}", db);

    println!("\n📊 Database Operations Demo:\n");

    // 1. Get all users
    println!("1️⃣  Get all users:");
    let query = db.get_rows().from("users");
    println!("   Query: {}", query);
    let all_users: Vec<User> = query.all().await;
    println!("   Found {} users", all_users.len());

    // 2. Find a specific user
    println!("\n2️⃣  Find user with id='2':");
    let user: Option<User> = db
        .get_single()
        .from("users")
        .where_eq("id", "2")
        .await?;
    if let Some(u) = user {
        println!("   Found: {} (age: {})", u.name, u.age);
    }

    // 3. Bulk update users
    println!("\n3️⃣  Bulk update all active users (increment age by 1):");
    let update_data = json!({
        "age": 29  // This is just for demo; in real usage you'd calculate new values
    });
    let updated_count = db
        .update_many()
        .from("users")
        .data(update_data)
        .where_equals("status", "active")
        .execute_bulk_update()
        .await?;
    println!("   Updated {} active users\n", updated_count);

    // Reload after update
    db.reload().await?;

    // 4. Count records
    println!("\n4️⃣  Count users in table:");
    let count = db.count_rows("users")?;
    println!("   Total users: {}", count);

    // 5. Check if record exists
    println!("\n5️⃣  Check if user exists:");
    let exists = db.record_exists("users", "1");
    println!("   User with id='1' exists: {}", exists);

    // 6. Advanced Query: Find users over 30
    println!("\n6️⃣  Find users over 30 years old:");
    let query = db
        .get_rows()
        .from("users")
        .where_gt("age", json!(30));
    println!("   Query: {}", query);
    let seniors: Vec<User> = query.all().await;
    println!("   Found {} users over 30:", seniors.len());
    for user in &seniors {
        println!("     - {} (age: {})", user.name, user.age);
    }

    // 7. Advanced Query: Active users between 25-40, sorted by age
    println!("\n7️⃣  Find active users aged 25-40, sorted by age:");
    let query = db
        .get_rows()
        .from("users")
        .where_gte("age", json!(25))
        .where_lte("age", json!(40))
        .order_by("age", cargobase::SortOrder::Asc);
    println!("   Query: {}", query);
    let active_mid_age: Vec<User> = query.all().await;
    println!("   Found {} users:", active_mid_age.len());
    for user in &active_mid_age {
        println!("     - {} (age: {}, status: {})", user.name, user.age, user.status);
    }

    // 8. Pagination: Get first 2 users ordered by name
    println!("\n8️⃣  Get first 2 users (paginated, ordered by name):");
    let query = db
        .get_rows()
        .from("users")
        .order_by("name", cargobase::SortOrder::Asc)
        .limit(2);
    println!("   Query: {}", query);
    let page1: Vec<User> = query.all().await;
    println!("   Page 1:");
    for user in &page1 {
        println!("     - {}", user.name);
    }

    // 9. Get next page
    println!("\n9️⃣  Get next 2 users (page 2):");
    let page2: Vec<User> = db
        .get_rows()
        .from("users")
        .order_by("name", cargobase::SortOrder::Asc)
        .offset(2)
        .limit(2)
        .all()
        .await;
    println!("   Page 2:");
    for user in &page2 {
        println!("     - {}", user.name);
    }

    // 10. Display table
    println!("\n🔟 Display users table:");
    db.reload().await?; // Ensure we have latest state
    if let Some(table) = db.tables.get("users") {
        println!("   Table info: {}", table);
    }
    db.view_table("users");

    // 11. List all tables
    println!("\n1️⃣1️⃣  List all tables:");
    let tables = db.list_tables();
    println!("   Tables: {:?}", tables);

    // 12. Bulk delete inactive users
    println!("\n1️⃣2️⃣  Bulk delete inactive users:");
    let deleted_count = db
        .delete_many()
        .from("users")
        .where_equals("status", "inactive")
        .execute_bulk_delete()
        .await?;
    println!("   Deleted {} inactive users", deleted_count);

    // Show final state
    println!("\n📊 Final table state:");
    db.reload().await?; // Reload to see the deleted state
    db.view_table("users");

    // Cleanup
    println!("\n🧹 Cleaning up...");
    db.drop_database().await?;
    println!("✅ Database deleted\n");

    println!("🎉 Demo completed successfully!");

    Ok(())
}
