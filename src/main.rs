use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::Deserialize;
use warp::{Filter, Reply, Rejection};

#[derive(Clone)]
struct AppState {
    // Use DashMap for concurrent, lock-free (optimistic) in-memory caching.
    memory: Arc<DashMap<String, CacheEntry>>,
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

#[tokio::main]
async fn main() {
    // Create shared state using DashMap.
    let state = Arc::new(AppState {
        memory: Arc::new(DashMap::new()),
    });

    // Define the set route.
    let set_route = warp::path("set")
        .and(warp::body::json())
        .and(with_state(state.clone()))
        .and_then(set_handler);

    // Define the get route.
    let get_route = warp::path("get")
        .and(warp::query::<GetQuery>())
        .and(with_state(state.clone()))
        .and_then(get_handler);

    let routes = set_route.or(get_route);

    println!("Server running on http://localhost:6060");
    warp::serve(routes).run(([127, 0, 0, 1], 6060)).await;
}

// Helper filter to inject shared application state.
fn with_state(state: Arc<AppState>) -> impl Filter<Extract = (Arc<AppState>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || state.clone())
}

// The "set" handler updates the in-memory cache immediately.
async fn set_handler(req: SetRequest, state: Arc<AppState>) -> Result<impl Reply, Rejection> {
    // Calculate expiration if a TTL is provided.
    let expires_at = req.ttl.map(|ttl| Instant::now() + Duration::from_secs(ttl));
    
    // Insert/update the in-memory cache.
    state.memory.insert(req.key.clone(), CacheEntry { 
        value: req.value, 
        expires_at 
    });
    
    Ok(warp::reply::json(&serde_json::json!({"status": "success"})))
}

// The "get" handler reads from the in-memory cache and checks for expiration.
async fn get_handler(query: GetQuery, state: Arc<AppState>) -> Result<impl Reply, Rejection> {
    let now = Instant::now();
    if let Some(entry) = state.memory.get(&query.key) {
        if let Some(exp) = entry.expires_at {
            if now > exp {
                state.memory.remove(&query.key);
                return Ok(warp::reply::json(&serde_json::json!({"error": "Key not found"})));
            }
        }
        return Ok(warp::reply::json(&serde_json::json!({
            "key": query.key,
            "value": entry.value.clone()
        })));
    }
    Ok(warp::reply::json(&serde_json::json!({"error": "Key not found"})))
}
