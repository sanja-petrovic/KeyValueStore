use std::{collections::HashMap, io};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::Duration;
use tokio::task;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Int(i64),
    Double(f64),
}

#[derive(Clone)]
pub struct KeyValueStore<K, Value> {
    data: HashMap<K, Value>,
}

impl<K: Eq + PartialEq + std::hash::Hash + std::fmt::Debug, Value: std::fmt::Debug> KeyValueStore<K, Value> {
    pub fn new() -> KeyValueStore<K, Value> {
        KeyValueStore {
            data: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: K, value: Value) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn print(&self) {
        for (key, value) in &self.data {
            println!("Key: {:?}, Value: {:?}", key, value);
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();
    let store = Arc::new(RwLock::new(KeyValueStore::new()));

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, Arc::clone(&store)));
    }

    println!("Shutting down.");
}

async fn handle_connection(mut stream: TcpStream, mut store: Arc<RwLock<KeyValueStore<String, Value>>>) {
    let mut buffer = [0; 1024];
    if let Err(e) = stream.read(&mut buffer).await {
        eprintln!("Error reading from stream: {}", e);
        return;
    }

    let get = b"GET /get?key=";
    let put = b"PUT /put?key=";
    let request_str = std::str::from_utf8(&buffer).unwrap_or("[Invalid UTF-8]");
    println!("{}", request_str);
    if buffer.starts_with(get) {
        let key_start = get.len();
        let key_end = buffer.iter().skip(get.len()).position(|&x| x == b' ').map(|pos| pos + key_start).unwrap_or(buffer.len());
        let key = std::str::from_utf8(&buffer[key_start..key_end]).unwrap_or("[Invalid UTF-8]");
        println!("Extracted key: {}", key);

        {
            let guard = store.read().await;
            if let Some(value) = guard.get(&key.to_string()) {
                let response = format!("HTTP/1.1 200 OK\r\n\r\n{:?}", value);
                if let Err(e) = stream.write_all(response.as_bytes()).await {
                    eprintln!("Error writing to stream: {}", e);
                }
            } else {
                let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
                if let Err(e) = stream.write_all(response.as_bytes()).await {
                    eprintln!("Error writing to stream: {}", e);
                }
            }
        }
    } else if buffer.starts_with(put) {
        let first_equals_sign_position = buffer.iter().position(|&x| x == b'=').unwrap_or(buffer.len());
        let ampersand_position = buffer.iter().skip(first_equals_sign_position + 1).position(|&x| x == b'&').map(|pos| pos + first_equals_sign_position + 1).unwrap_or(buffer.len());
        let key_vec: Vec<u8> = buffer[first_equals_sign_position + 1..ampersand_position].to_vec();

        let key_str = String::from_utf8_lossy(&key_vec);

        let second_equals_sign_position = buffer.iter().skip(first_equals_sign_position + 1).position(|&x| x == b'=').map(|pos| pos + first_equals_sign_position + 1).unwrap_or(buffer.len());
        let second_space_position = buffer.iter().skip(second_equals_sign_position + 1).position(|&x| x == b' ').map(|pos| pos + second_equals_sign_position + 1).unwrap_or(buffer.len());
        
        let value_vec: Vec<u8> = buffer[second_equals_sign_position + 1..second_space_position].to_vec();
        let value_str = String::from_utf8_lossy(&value_vec);
        println!("Extracted value: {}", value_str);
        let value = match value_str.parse::<i64>() {
            Ok(int_val) => Value::Int(int_val),
            Err(_) => match value_str.parse::<f64>() {
                Ok(double_val) => Value::Double(double_val),
                Err(_) => Value::String(value_str.to_string()),
            },
        };
        println!("{:?}", value);

        {     
            let mut guard = store.write().await;
            guard.put(key_str.to_string(), value);
        }

        let response = "HTTP/1.1 200 OK\r\n\r\n";
        if let Err(e) = stream.write_all(response.as_bytes()).await {
            eprintln!("Error writing to stream: {}", e);
        }
    } else {
        let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
        if let Err(e) = stream.write_all(response.as_bytes()).await {
            eprintln!("Error writing to stream: {}", e);
        }
    }

    if let Err(e) = stream.flush().await {
        eprintln!("Error flushing stream: {}", e);
    }
}
