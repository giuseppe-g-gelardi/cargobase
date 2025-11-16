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

    // Insert some sample users
    println!("📝 Inserting sample users...");
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

    for user in &users {
        db.add_row()
            .from("users")
            .data_from_struct(user.clone())
            .execute_add()
            .await?;
        println!("  ➕ Added user: {}", user.name);
    }

    // Reload to sync in-memory state
    db.reload().await?;

    println!("\n📊 Database Operations Demo:\n");

    // 1. Get all users
    println!("1️⃣  Get all users:");
    let all_users: Vec<User> = db.get_rows().from("users").all().await;
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

    // 3. Update a user
    println!("\n3️⃣  Update user with id='3':");
    let update_data = json!({
        "status": "active",
        "age": 43
    });
    let updated: Option<User> = db
        .update_row()
        .from("users")
        .data(update_data)
        .where_eq("id", "3")
        .await?;
    if let Some(u) = updated {
        println!("   Updated: {} - new status: {}, new age: {}", u.name, u.status, u.age);
    }

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
    let seniors: Vec<User> = db
        .get_rows()
        .from("users")
        .where_gt("age", json!(30))
        .all()
        .await;
    println!("   Found {} users over 30:", seniors.len());
    for user in &seniors {
        println!("     - {} (age: {})", user.name, user.age);
    }

    // 7. Advanced Query: Active users between 25-40, sorted by age
    println!("\n7️⃣  Find active users aged 25-40, sorted by age:");
    let active_mid_age: Vec<User> = db
        .get_rows()
        .from("users")
        .where_gte("age", json!(25))
        .where_lte("age", json!(40))
        .order_by("age", cargobase::SortOrder::Asc)
        .all()
        .await;
    println!("   Found {} users:", active_mid_age.len());
    for user in &active_mid_age {
        println!("     - {} (age: {}, status: {})", user.name, user.age, user.status);
    }

    // 8. Pagination: Get first 2 users ordered by name
    println!("\n8️⃣  Get first 2 users (paginated, ordered by name):");
    let page1: Vec<User> = db
        .get_rows()
        .from("users")
        .order_by("name", cargobase::SortOrder::Asc)
        .limit(2)
        .all()
        .await;
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
    db.view_table("users");

    // 11. List all tables
    println!("\n1️⃣1️⃣  List all tables:");
    let tables = db.list_tables();
    println!("   Tables: {:?}", tables);

    // 12. Delete a user
    println!("\n1️⃣2️⃣  Delete user with id='4':");
    let deleted: Option<User> = db
        .delete_single()
        .from("users")
        .where_eq("id", "4")
        .await?;
    if let Some(u) = deleted {
        println!("   Deleted: {}", u.name);
    }

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
