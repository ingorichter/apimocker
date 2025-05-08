mod state;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use std::{collections::HashMap, fs::File, io::Write};
use std::sync::Arc;
use clap::Parser;
use tokio::sync::RwLock;
use crate::state::Db;

#[derive(Parser, Debug)]
#[command(name = "apimocker")]
#[command(author = "Ingo Richter")]
#[command(version = "1.0")]
#[command(about = "Mocks REST endpoints from a JSON file and creates CRUD operations", long_about = None)]
struct Args {
    /// Path to the JSON data file
    #[arg(short, long, required = true)]
    file: String,
}

#[derive(Clone)]
struct AppState {
    db: Db,
    file_path: String,
}

async fn write_to_disk(file_path: &str, db: &HashMap<String, Vec<Value>>) {
    if let Ok(json) = serde_json::to_string_pretty(db) {
        if let Ok(mut file) = File::create(file_path) {
            let _ = file.write_all(json.as_bytes());
        }
    }
}

fn find_by_id(list: &[Value], id: &str) -> Option<usize> {
    list.iter()
        .position(|item| {
            if let Some(entry) = item.get("id") {
                // id's can be numbers or strings. So, convert the id to string
                return *entry.to_string() == *id
            }
            false
        })
}

// GET /api/:route
async fn get_all(
    Path(route): Path<String>,
    State(state): State<AppState>,
) -> Json<Vec<Value>> {
    let db = state.db.read().await;
    Json(db.get(&route).cloned().unwrap_or_default())
}

// GET /api/:route/:id
async fn get_by_id(
    Path((route, id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Json<Value> {
    let db = state.db.read().await;
    if let Some(list) = db.get(&route) {
        if let Some(idx) = find_by_id(list, &id) {
            return Json(list[idx].clone());
        }
    }

    Json(json!({"error": "Not found"}))
}

// PUT /api/:route/:id
async fn replace(
    Path((route, id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut db = state.db.write().await;
    if let Some(list) = db.get_mut(&route) {
        if let Some(idx) = find_by_id(list, &id) {
            let mut new_body = body.clone();

            let index_type = list[idx]["id"].clone();

            if index_type.is_string() {
                new_body["id"] = json!(id);
            } else if index_type.is_number() {
                new_body["id"] = json!(id.parse::<i32>().unwrap());
            } else {
                return Json(json!({"error": "Invalid ID type"}));
            }

            list[idx] = new_body.clone();
            return Json(new_body);
        }
    }

    write_to_disk(&state.file_path, &db).await;

    Json(json!({"error": "Not found"}))}

// PATCH /api/:route/:id
async fn update(
    Path((route, id)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(patch): Json<Value>,
) -> Json<Value> {
    let mut db = state.db.write().await;
    if let Some(list) = db.get_mut(&route) {
        if let Some(idx) = find_by_id(list, &id) {
            if let Some(obj) = list[idx].as_object_mut() {
                for (k, v) in patch.as_object().unwrap() {
                    obj.insert(k.clone(), v.clone());
                }
                return Json(json!(obj));
            }
        }
    }

    write_to_disk(&state.file_path, &db).await;

    Json(json!({"error": "Not found"}))
}

// DELETE /api/:route/:id
async fn delete(
    Path((route, id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Json<Value> {
    let mut db = state.db.write().await;
    if let Some(list) = db.get_mut(&route) {
        if let Some(idx) = find_by_id(list, &id) {
            let removed = list.remove(idx);
            return Json(removed);
        }
    }

    write_to_disk(&state.file_path, &db).await;

    Json(json!({"error": "Not found"}))
}

// POST /api/:route
async fn create(
    Path(route): Path<String>,
    State(state): State<AppState>,
    Json(value): Json<Value>,
) -> Json<Value> {
    let mut db = state.db.write().await;
    let entry = db.entry(route).or_insert_with(Vec::new);
    entry.push(value.clone());

    write_to_disk(&state.file_path, &db).await;

    Json(value)
}

#[tokio::main]
async fn main() {
    let args = Args::parse(); // Parse CLI args

    let file_data = std::fs::read_to_string(&args.file)
        .expect("Failed to read JSON file");
    let parsed: HashMap<String, Vec<Value>> = serde_json::from_str(&file_data)
        .expect("Failed to parse JSON");

    let state = AppState {
        db: Arc::new(RwLock::new(parsed)),
        file_path: args.file,
    };

    let app = Router::new()
        .route("/api/{route}", get(get_all).post(create))
        .route("/api/{route}/{id}", get(get_by_id).put(replace).patch(update).delete(delete))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}