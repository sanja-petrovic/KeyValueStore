use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

type Key = String;
type Value = String;
type Database = Arc<Mutex<HashMap<Key, Value>>>;

async fn handle_request(req: Request<Body>, db: Database) -> Result<Response<Body>, hyper::Error> {
    let (method, uri, _, _, _) = req.into_parts();
    let response = match (method, uri.path()) {
        (hyper::Method::GET, "/get") => {
            if let Some(key) = uri.query() {
                match db.lock().unwrap().get(key) {
                    Some(value) => Response::new(Body::from(value.clone())),
                    None => Response::builder()
                        .status(404)
                        .body(Body::from("Key not found"))
                        .unwrap(),
                }
            } else {
                Response::builder()
                    .status(400)
                    .body(Body::from("Invalid request"))
                    .unwrap()
            }
        }
        (hyper::Method::PUT, "/put") => {
            let chunks = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let key_value: Vec<&str> = chunks.split(|&b| b == b'=').map(|s| unsafe { String::from_utf8_unchecked(s.to_vec()) }).collect();
            if key_value.len() == 2 {
                let (key, value) = (key_value[0].clone(), key_value[1].clone());
                db.lock().unwrap().insert(key, value);
                Response::new(Body::from("Key-value pair inserted"))
            } else {
                Response::builder()
                    .status(400)
                    .body(Body::from("Invalid request"))
                    .unwrap()
            }
        }
        _ => Response::builder()
            .status(404)
            .body(Body::from("Not Found"))
            .unwrap(),
    };

    Ok(response)
}

#[tokio::main]
async fn main() {
    let addr = ([127, 0, 0, 1], 8080).into();
    let db = Arc::new(Mutex::new(HashMap::new()));

    let make_svc = make_service_fn(|_conn| {
        let db = db.clone();
        async { Ok::<_, hyper::Error>(service_fn(move |req| handle_request(req, db.clone()))) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
