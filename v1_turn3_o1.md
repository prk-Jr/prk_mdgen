<file> Cargo.toml </file>  
<code>  
[package]  
name = "trait_enforcement_demo"  
version = "0.1.0"  
edition = "2021"  

[dependencies]  
anyhow = "1.0"  
tokio = { version = "1", features = ["macros"] }  
sqlx = { version = "0.6", features = ["sqlite", "runtime-tokio", "macros"] }  
async-trait = "0.1"  
</code>  

<file> src/main.rs </file>  
<code>  
use anyhow::Result;  
use trait_enforcement_demo::{  
    aggregator::DataAggregator,  
    storage::{db::DatabaseStorage, file::FileStorage},  
    traits::{Displayable, Showable},  
    MyStruct, User,  
};  

#[tokio::main]  
async fn main() -> Result<()> {  
    // -- Database usage  
    let mut db_storage = DatabaseStorage::new(":memory:").await?;  
    db_storage.init_schema().await?;  
    let mut db_aggregator = DataAggregator {  
        storage: db_storage,  
    };  
    let user1 = User {  
        id: 0,  
        name: "AliceDB".to_string(),  
        age: 35,  
    };  
    db_aggregator.add_user(user1).await?;  

    // -- File usage  
    let mut file_storage = FileStorage::new("users.txt");  
    let mut file_aggregator = DataAggregator {  
        storage: file_storage,  
    };  
    let user2 = User {  
        id: 1,  
        name: "BobFile".to_string(),  
        age: 22,  
    };  
    file_aggregator.add_user(user2).await?;  

    // -- Demonstrate trait collisions  
    let example = MyStruct { data: 999 };  
    println!("Displayable call: {}", example.display());  
    println!("Showable   call: {}", Showable::display(&example));  

    Ok(())  
}
</code>  

<file> src/lib.rs </file>  
<code>  
pub mod aggregator;  
pub mod storage;  
pub mod traits;  

// A simple user type for real storage usage
#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub age: i64,
}

// A type implementing both Displayable and Showable
pub struct MyStruct {
    pub data: i32,
}

use crate::traits::{Displayable, Showable};

impl Displayable for MyStruct {
    fn display(&self) -> String {
        format!("(Displayable) data = {}", self.data)
    }
}

impl Showable for MyStruct {
    fn display(&self) -> String {
        format!("(Showable) data + 100 = {}", self.data + 100)
    }
}
</code>  

<file> src/traits.rs </file>
<code>
use async_trait::async_trait;

// A super-trait
pub trait SomeSuperTrait {
    fn super_trait_name(&self) -> &'static str {
        "SomeSuperTrait"
    }
}

// Storage extends the super-trait
#[async_trait]
pub trait Storage: SomeSuperTrait {
    type Item;
    async fn store(&mut self, item: Self::Item) -> anyhow::Result<()>;
}

// Trait 1 - display
pub trait Displayable {
    fn display(&self) -> String;
}

// Trait 2 - also display
pub trait Showable {
    fn display(&self) -> String;
}
</code>

<file> src/aggregator.rs </file>  
<code>  
use crate::{User};  
use crate::traits::Storage;
use anyhow::Result;

pub struct DataAggregator<S: Storage<Item = User>> {
    pub storage: S,
}

impl<S: Storage<Item = User>> DataAggregator<S> {
    pub async fn add_user(&mut self, user: User) -> Result<()> {
        self.storage.store(user).await?;
        Ok(())
    }
}
</code>  

<file> src/storage/db.rs </file>  
<code>  
use async_trait::async_trait;
use sqlx::{SqlitePool, sqlite::SqliteQueryResult};
use anyhow::Result;
use crate::traits::{SomeSuperTrait, Storage};
use crate::User;

pub struct DatabaseStorage {
    pub pool: SqlitePool,
}

impl DatabaseStorage {
    pub async fn new(db_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(db_url).await?;
        Ok(Self { pool })
    }

    pub async fn init_schema(&mut self) -> Result<()> {
        sqlx::query("
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                age INTEGER NOT NULL
            );
        ")
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

impl SomeSuperTrait for DatabaseStorage {}

#[async_trait]
impl Storage for DatabaseStorage {
    type Item = User;

    async fn store(&mut self, item: User) -> Result<()> {
        let _res: SqliteQueryResult = sqlx::query(
            "INSERT INTO users (name, age) VALUES (?, ?)"
        )
        .bind(item.name)
        .bind(item.age)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
</code>  

<file> src/storage/file.rs </file>  
<code>  
use async_trait::async_trait;
use crate::{
    traits::{SomeSuperTrait, Storage},
    User,
};
use anyhow::Result;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

pub struct FileStorage {
    pub file_path: String,
}

impl FileStorage {
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_owned(),
        }
    }
}

impl SomeSuperTrait for FileStorage {}

#[async_trait]
impl Storage for FileStorage {
    type Item = User;

    async fn store(&mut self, item: User) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .await?;
        let line = format!("Storing user: name={}, age={}\n", item.name, item.age);
        file.write_all(line.as_bytes()).await?;
        Ok(())
    }
}
</code>  

<file> src/storage/mod.rs </file>  
<code>  
pub mod db;  
pub mod file;  
</code>  

<file> tests/integration_test.rs </file>
<code>
use anyhow::Result;
use trait_enforcement_demo::{
    aggregator::DataAggregator,
    storage::{db::DatabaseStorage, file::FileStorage},
    User,
};
use std::fs;
use tokio::runtime::Runtime;

#[test]
fn test_database_storage() -> Result<()> {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut storage = DatabaseStorage::new(":memory:").await?;
        storage.init_schema().await?;
        let mut aggregator = DataAggregator { storage };
        aggregator.add_user(User {
            id: 0,
            name: "TestDB".to_string(),
            age: 99,
        }).await?;

        // Perform a quick check to ensure row is inserted
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&aggregator.storage.pool)
            .await?;
        assert_eq!(row.0, 1);

        Ok(())
    })
}

#[test]
fn test_file_storage() -> Result<()> {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        // Use a test file
        let file_path = "test_users.txt";
        // Clean up before
        let _ = fs::remove_file(file_path);

        let mut storage = FileStorage::new(file_path);
        let mut aggregator = DataAggregator { storage };
        aggregator.add_user(User {
            id: 1,
            name: "TestFile".to_string(),
            age: 42,
        }).await?;

        // read from file to check
        let contents = fs::read_to_string(file_path)?;
        assert!(contents.contains("TestFile"));
        assert!(contents.contains("42"));

        // Clean up after
        let _ = fs::remove_file(file_path);

        Ok(())
    })
}
</code>

## Run Instructions

1) Build:  
   cargo build  

2) Run:  
   cargo run  

3) Test:  
   cargo test  

## Expected Output (cargo run)

• It should create and initialize an in-memory SQLite database, insert "AliceDB" user.  
• It should create or append to a file named "users.txt" with "BobFile" user info.  
• Then it should print the Displayable and Showable versions of MyStruct.  

Something like:

(DatabaseStorage) storing user with ID=...  
...  
Storing user: name=BobFile, age=22  
Displayable call: (Displayable) data = 999  
Showable   call: (Showable) data + 100 = 1099  

## Expected Output (cargo test)

All tests should pass, showing the aggregator can store a user in the DB and file.  

running 2 tests  
test integration_test::test_database_storage ... ok  
test integration_test::test_file_storage ... ok  

test result: ok. 2 passed; 0 failed.  