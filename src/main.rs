mod state;

use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::get;
use serde_json::Value;
use tokio::sync::RwLock;
use crate::state::Db;

pub async fn get_all(
    Path(route): Path<String>,
    State(db): State<Db>,
) -> Json<Vec<Value>> {
    let db = db.read().await;
    Json(db.get(&route).cloned().unwrap_or_default())
}

#[tokio::main]
async fn main() {
    let json = std::fs::read_to_string("examples/data.json").unwrap();
    let parsed: HashMap<String, Vec<Value>> = serde_json::from_str(&json).unwrap();

    let db: Db = Arc::new(RwLock::new(parsed));

    let app = Router::new()
        .route("/api/{route}", get(get_all))
        .with_state(db);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}