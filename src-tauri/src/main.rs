#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use sqlx::migrate::MigrateDatabase;
use sqlx::SqlitePool;
use std::path::PathBuf;
use tauri::{Manager, State};

pub(crate) mod database;

#[derive(Debug, Serialize, Deserialize)]
pub struct Board {
    columns: Vec<Column>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    id: i64,
    title: String,
    cards: Vec<Card>,
}

impl Column {
    pub fn new(id: i64, title: &str) -> Self {
        Column {
            id,
            title: title.to_string(),
            cards: Vec::new(),
        }
    }

    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    id: i64,
    title: String,
    description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    id: i64,
    title: String,
    content: String,
}

#[tauri::command]
fn greet() -> String {
    const DATABASE_DIR: &str = "doit-db";
    const DATABASE_FILE: &str = "db.sqlite";
    let home_dir = directories::UserDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        // ホームディレクトリが取得できないときはカレントディレクトリを使う
        .unwrap_or_else(|| std::env::current_dir().expect("Cannot access the current directory"));
    let database_dir = home_dir.join(DATABASE_DIR);
    let database_dir_str = dunce::canonicalize(&database_dir)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    let database_url = format!("sqlite://{}/{}", database_dir_str, DATABASE_FILE);

    format!("Hello, {}! You've been greeted from Rust!", database_url)
}

// #[tauri::command]
// async fn get_board(sqlite_pool: State<'_, sqlx::SqlitePool>) -> Result<Board, String> {
//     let columns = database::get_columns(&*sqlite_pool)
//         .await
//         .map_err(|e| e.to_string())?;
//     Ok(Board { columns })
// }

#[tauri::command]
async fn handle_add_note(
    sqlite_pool: State<'_, sqlx::SqlitePool>,
    // note: Note,
) -> Result<(), String> {
    println!("Hello World!");
    let note = {
        Note {
            id: 1,
            title: "title".to_string(),
            content: "content".to_string(),
        }
    };
    database::insert_note(&*sqlite_pool, note)
        .await
        .map_err(|e| e.to_string())?;

    println!("note added");
    Ok(())
}

// #[tauri::command]
// async fn handle_move_card(
//     sqlite_pool: State<'_, sqlx::SqlitePool>,
//     card: Card,
//     from: CardPos,
//     to: CardPos,
// ) -> Result<(), String> {
//     database::move_card(&*sqlite_pool, card, from, to)
//         .await
//         .map_err(|e| e.to_string())?;

//     Ok(())
// }

// #[tauri::command]
// async fn handle_remove_card(
//     sqlite_pool: State<'_, sqlx::SqlitePool>,
//     card: Card,
//     column_id: i64,
// ) -> Result<(), String> {
//     database::delete_card(&*sqlite_pool, card, column_id)
//         .await
//         .map_err(|e| e.to_string())?;
//     Ok(())
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("main");
    // このmain関数はasync fnではないので、asyncな関数を呼ぶのにblock_on関数を使う
    use tauri::async_runtime::block_on;

    let db_url: &str = "sqlite:database.db";

    let db_exists = block_on(database::check_database_exists(&db_url))?;

    // SQLiteのコネクションプールを作成する
    let sqlite_pool = block_on(database::create_sqlite_pool(&db_url))?;

    print!("db_exists: {}", db_exists);
    if !db_exists {
        println!("db_exists");
        block_on(database::migrate_database(&sqlite_pool))?;
        println!("db_exists down");
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            handle_add_note,
            // get_board,
            // handle_add_card,
            // handle_move_card,
            // handle_remove_card
        ])
        // ハンドラからコネクションプールにアクセスできるよう、登録する
        .setup(|app| {
            app.manage(sqlite_pool);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    println!("main down");
    Ok(())
}
