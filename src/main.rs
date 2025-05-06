mod state;

use crate::state::Db;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

fn find_by_id(list: &[Value], id: &str) -> Option<usize> {
    list.iter()
        .position(|item| {
            if let Some(entry) = item.get("id") {
                // id's can be numbers or strings. So, convert the id to string
                return entry.to_string() == id
            }
            false
        })
}

// GET /api/:route
pub async fn get_all(
    Path(route): Path<String>,
    State(db): State<Db>,
) -> Json<Vec<Value>> {
    let db = db.read().await;
    Json(db.get(&route).cloned().unwrap_or_default())
}

// GET /api/:route/:id
pub async fn get_by_id(
    Path((route, id)): Path<(String, String)>,
    State(db): State<Db>,
) -> Json<Value> {
    let db = db.read().await;
    if let Some(list) = db.get(&route) {
        if let Some(idx) = find_by_id(list, &id) {
            return Json(list[idx].clone());
        }
    }

    Json(json!({"error": "Not found"}))
}

// PUT /api/:route/:id
pub async fn replace(
    Path((route, id)): Path<(String, String)>,
    State(db): State<Db>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut db = db.write().await;
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
    Json(json!({"error": "Not found"}))}

// PATCH /api/:route/:id
pub async fn update(
    Path((route, id)): Path<(String, String)>,
    State(db): State<Db>,
    Json(patch): Json<Value>,
) -> Json<Value> {
    let mut db = db.write().await;
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
    Json(json!({"error": "Not found"}))
}

// DELETE /api/:route/:id
pub async fn delete(
    Path((route, id)): Path<(String, String)>,
    State(db): State<Db>,
) -> Json<Value> {
    let mut db = db.write().await;
    if let Some(list) = db.get_mut(&route) {
        if let Some(idx) = find_by_id(list, &id) {
            let removed = list.remove(idx);
            return Json(removed);
        }
    }
    Json(json!({"error": "Not found"}))
}

// POST /api/:route
pub async fn create(
    Path(route): Path<String>,
    State(db): State<Db>,
    Json(value): Json<Value>,
) -> Json<Value> {
    let mut db = db.write().await;
    let entry = db.entry(route).or_insert_with(Vec::new);
    entry.push(value.clone());
    Json(value)
}

#[tokio::main]
async fn main() {
    let json = std::fs::read_to_string("examples/data.json").unwrap();
    let parsed: HashMap<String, Vec<Value>> = serde_json::from_str(&json).unwrap();

    let db: Db = Arc::new(RwLock::new(parsed));

    let app = Router::new()
        .route("/api/{route}", get(get_all).post(create))
        .route("/api/{route}/{id}", get(get_by_id).put(replace).patch(update).delete(delete))
        .with_state(db);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}