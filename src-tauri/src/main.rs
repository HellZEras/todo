#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use rusqlite::{params, Connection, Error, Result};
use serde_json::json;
use tauri::{CustomMenuItem, Manager, State, SystemTray, SystemTrayEvent, SystemTrayMenu};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Task {
    pub id: usize,
    pub task: String,
}

struct DatabaseConnection {
    conn: Mutex<Connection>,
}

fn connect_to_db() -> Result<Connection> {
    Connection::open("tasks.db")
}

fn create_db(conn:&Connection) -> Result<usize,Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY,
            task TEXT NOT NULL
        )",
        (),
    ) 
}

#[tauri::command]
fn add_task_to_db(state: State<DatabaseConnection>,task_to_add: &str) ->String   {
        match state.conn.lock().unwrap().execute(
            "INSERT INTO tasks (task) VALUES (?1)",
            params![task_to_add],
        ) {
            Ok(_) => "Success".to_string(),
            Err(e) => e.to_string(),
        }
    }
    

#[tauri::command]
fn remove_task_from_db(state: State<DatabaseConnection>, id: usize) -> String {
    let mut conn = state.conn.lock().unwrap();
    let tx = conn.transaction().unwrap();

    match tx.execute(
        "DELETE FROM tasks WHERE id = ?1",
        params![id],
    ) {
        Ok(_) => {
            let reorder_sql = "
                WITH OrderedTasks AS (
                    SELECT id, ROW_NUMBER() OVER (ORDER BY id) AS new_id
                    FROM tasks
                )
                UPDATE tasks
                SET id = (SELECT new_id FROM OrderedTasks WHERE OrderedTasks.id = tasks.id);
            ";
            match tx.execute_batch(reorder_sql) {
                Ok(_) => {
                    tx.commit().unwrap();
                    "Success".to_string()
                },
                Err(e) => e.to_string(),
            }
        },
        Err(e) => e.to_string(),
    }
}

#[tauri::command]
fn get_all_tasks(state: State<DatabaseConnection>)-> String{
    let conn = state.conn.lock().map_err(|_| "Failed to acquire lock".to_string()).unwrap();
    let mut stmt = conn.prepare("SELECT * FROM tasks").map_err(|e| e.to_string()).unwrap();
    let task_iter = stmt.query_map(params![], |row| {
        Ok(Task {
            id: row.get(0)?,
            task: row.get(1)?,
        })
    }).map_err(|e| e.to_string()).unwrap();

    let tasks: Result<Vec<_>, _> = task_iter.collect();
    json!(tasks.unwrap()).to_string()
}

fn main() {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("hide".to_string(), "Hide"))
        .add_item(CustomMenuItem::new("show".to_string(), "Show"))
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));
    let system_tray = SystemTray::new()
    .with_menu(tray_menu);
    let conn = connect_to_db().expect("Failed to connect to database");
    create_db(&conn).expect("Couldnt create db");
    tauri::Builder::default()
        .manage(DatabaseConnection {
            conn: Mutex::new(conn),
        })
        .invoke_handler(tauri::generate_handler![
            add_task_to_db,
            remove_task_from_db,
            get_all_tasks
        ])
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| {
            if let SystemTrayEvent::MenuItemClick { id, .. } = event {
                match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "hide" => {
                        let window = app.get_window("main").unwrap();
                        window.hide().unwrap();
                    }
                    "show" => {
                        let window = app.get_window("main").unwrap();
                        window.show().unwrap();
                    }
                    _ => {
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
