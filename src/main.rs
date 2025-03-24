use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rocksdb::{DB, Options};
use dashmap::DashMap;
use bytes::BytesMut;
use std::sync::Arc;

struct Cache {
    memory: Arc<DashMap<String, String>>,
    db: Option<Arc<DB>>,
}

impl Cache {
    fn new(use_rocksdb: bool) -> Self {
        let db = if use_rocksdb {
            let mut opts = Options::default();
            opts.create_if_missing(true);
            Some(Arc::new(DB::open(&opts, "rocksdb-data").unwrap()))
        } else {
            None
        };

        Cache {
            memory: Arc::new(DashMap::new()),
            db,
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        self.memory.get(key).map(|v| v.to_string()).or_else(|| {
            self.db.as_ref()
                .and_then(|db| db.get(key).ok().flatten())
                .and_then(|v| String::from_utf8(v).ok())
        })
    }

    fn set(&self, key: &str, value: &str) {
        self.memory.insert(key.to_string(), value.to_string());
        if let Some(db) = &self.db {
            let _ = db.put(key, value);
        }
    }
}

async fn handle_client(mut stream: TcpStream, cache: Arc<Cache>) {
    let mut buf = BytesMut::with_capacity(1024);

    loop {
        buf.clear();
        let n = match stream.read_buf(&mut buf).await {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };

        let request = match std::str::from_utf8(&buf[..n]) {
            Ok(s) => s.trim(),
            Err(_) => {
                let _ = stream.write_all(b"ERROR: Invalid UTF-8\n").await;
                continue;
            }
        };

        let mut parts = request.split_whitespace();
        let cmd = parts.next();
        let key = parts.next();
        let val = parts.next();

        let response = match (cmd, key, val) {
            (Some("GET"), Some(k), None) => cache.get(k).unwrap_or_else(|| "Key not found".into()),
            (Some("SET"), Some(k), Some(v)) => {
                cache.set(k, v);
                "OK".into()
            }
            _ => "ERROR: Invalid command".into(),
        };

        if stream.write_all(response.as_bytes()).await.is_err() {
            break;
        }
        let _ = stream.flush().await;
    }
}

#[tokio::main]
async fn main() {
    let use_rocksdb = std::env::var("USE_ROCKSDB").unwrap_or_default().to_lowercase() == "true";
    let cache = Arc::new(Cache::new(use_rocksdb));
    let listener = TcpListener::bind("0.0.0.0:6060").await.unwrap();

    loop {
        if let Ok((stream, _)) = listener.accept().await {
            let cache = cache.clone();
            tokio::spawn(async move {
                handle_client(stream, cache).await;
            });
        }
    }
}
