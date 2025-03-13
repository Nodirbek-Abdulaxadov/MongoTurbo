use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use mongodb::{
    bson::{doc, Bson, Document},
    Client,
};
use serde::Deserialize;
use tokio::sync::RwLock;
use warp::{Filter, Reply, Rejection, reject};

#[derive(Clone)]
struct AppState {
    client: Client,
    // In-memory cache: maps key to a CacheEntry (value + expiration time).
    memory: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    value: String,
    expires_at: Option<Instant>,
}

#[derive(Debug, Deserialize)]
struct SetRequest {
    key: String,
    value: String,
    ttl: Option<u64>, // TTL in seconds
}

#[derive(Debug, Deserialize)]
struct GetQuery {
    key: String,
}

// Custom error type for Warp rejections.
#[derive(Debug)]
struct DatabaseError;
impl reject::Reject for DatabaseError {}

#[tokio::main]
async fn main() {
    // Read MongoDB URI from environment or default to localhost.
    let mongo_uri = env::var("MONGO_URI")
        .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let client = Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to connect to MongoDB");

    let state = Arc::new(AppState {
        client,
        memory: Arc::new(RwLock::new(HashMap::new())),
    });

    let set_route = warp::path("set")
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(set_handler);

    let get_route = warp::path("get")
        .and(warp::query::<GetQuery>())
        .and(with_state(state.clone()))
        .and_then(get_handler);

    let routes = set_route.or(get_route);

    println!("Server running on http://localhost:6060");
    warp::serve(routes).run(([127, 0, 0, 1], 6060)).await;
}

// Helper filter to inject shared application state.
fn with_state(
    state: Arc<AppState>,
) -> impl Filter<Extract = (Arc<AppState>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

// The "set" handler updates the memory cache immediately and then spawns a background task
// to write the same data into MongoDB.
async fn set_handler(req: SetRequest, state: Arc<AppState>) -> Result<impl Reply, Rejection> {
    // Calculate expiration if a TTL is provided.
    let expires_at = req.ttl.map(|ttl| Instant::now() + Duration::from_secs(ttl));

    // Update in-memory cache.
    {
        let mut cache = state.memory.write().await;
        cache.insert(
            req.key.clone(),
            CacheEntry {
                value: req.value.clone(),
                expires_at,
            },
        );
    }

    // Spawn a background task to write to MongoDB.
    let state_clone = state.clone();
    let key_clone = req.key.clone();
    let value_clone = req.value.clone();
    let ttl_clone = req.ttl;
    tokio::spawn(async move {
        let collection: mongodb::Collection<Document> =
            state_clone.client.database("cache").collection("data");

        // If a TTL is provided, calculate the expiration datetime.
        let expires_at_bson = ttl_clone.map(|ttl| {
            let expiration = Utc::now() + chrono::Duration::seconds(ttl as i64);
            // Convert chrono::DateTime<Utc> into mongodb::bson::DateTime using milliseconds.
            Bson::DateTime(mongodb::bson::DateTime::from_millis(expiration.timestamp_millis()))
        }).unwrap_or(Bson::Null);

        let document = doc! {
            "key": key_clone,
            "value": value_clone,
            "expires_at": expires_at_bson,
        };

        // Fire-and-forget: ignore any error.
        let _ = collection.insert_one(document, None).await;
    });

    Ok(warp::reply::json(&doc! {"status": "success"}))
}

// The "get" handler reads from the in-memory cache and checks if the entry has expired.
async fn get_handler(query: GetQuery, state: Arc<AppState>) -> Result<impl Reply, Rejection> {
    let now = Instant::now();
    let maybe_entry = {
        let cache = state.memory.read().await;
        cache.get(&query.key).cloned()
    };

    if let Some(entry) = maybe_entry {
        // Check if the entry is expired.
        if let Some(exp) = entry.expires_at {
            if now > exp {
                let mut cache = state.memory.write().await;
                cache.remove(&query.key);
                return Ok(warp::reply::json(&doc! {"error": "Key not found"}));
            }
        }
        return Ok(warp::reply::json(&doc! {"key": query.key, "value": entry.value}));
    }

    Ok(warp::reply::json(&doc! {"error": "Key not found"}))
}
