use std::collections::HashMap;
use serde_json::Value;
use tokio::sync::RwLock;
use std::sync::Arc;

pub type Db = Arc<RwLock<HashMap<String, Vec<Value>>>>;