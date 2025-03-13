use std::env;
use std::sync::Arc;
use warp::{Filter, Reply, Rejection, reject};
use mongodb::{Client, bson::{doc, Bson, Document}};
use serde::Deserialize;

#[derive(Clone)]
struct CacheDB {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct SetRequest {
    key: String,
    value: String,
    ttl: Option<u64>,
}

// Custom error type for Warp rejection
#[derive(Debug)]
struct DatabaseError;
impl reject::Reject for DatabaseError {}

#[tokio::main]
async fn main() {
    let mongo_uri = env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let client = Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to connect to MongoDB");
    
    let db = Arc::new(CacheDB { client });

    let set_route = warp::path("set")
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(set_handler);

    let get_route = warp::path("get")
        .and(warp::query::<GetQuery>())
        .and(with_db(db.clone()))
        .and_then(get_handler);

    let routes = set_route.or(get_route);

    println!("Server running on http://localhost:6060");
    warp::serve(routes).run(([127, 0, 0, 1], 6060)).await;
}

#[derive(Debug, Deserialize)]
struct GetQuery {
    key: String,
}

fn with_db(db: Arc<CacheDB>) -> impl Filter<Extract = (Arc<CacheDB>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn set_handler(req: SetRequest, db: Arc<CacheDB>) -> Result<impl Reply, Rejection> {
    let collection: mongodb::Collection<Document> = db.client.database("cache").collection("data");

    let document = doc! {
        "key": &req.key,
        "value": &req.value,
        "expires_at": req.ttl.map(|ttl| Bson::Int64(ttl as i64)).unwrap_or(Bson::Null),
    };

    match collection.insert_one(document, None).await {
        Ok(_) => Ok(warp::reply::json(&doc! {"status": "success"})),
        Err(_) => Err(reject::custom(DatabaseError)),  // Fixed rejection type
    }
}

async fn get_handler(query: GetQuery, db: Arc<CacheDB>) -> Result<impl Reply, Rejection> {
    let key = query.key;  // Extract the key from the query struct

    let collection: mongodb::Collection<Document> = db.client.database("cache").collection("data");

    match collection.find_one(doc! {"key": &key}, None).await {
        Ok(Some(doc)) => Ok(warp::reply::json(&doc)),
        Ok(None) => Ok(warp::reply::json(&doc! {"error": "Key not found"})),
        Err(_) => Err(reject::custom(DatabaseError)),
    }
}
