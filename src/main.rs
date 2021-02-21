use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use http_types::{Response, StatusCode};

mod web {
    include!(concat!(env!("OUT_DIR"), "/web.rs"));
}

#[async_std::main]
async fn main() -> http_types::Result<()> {
    // Open up a TCP connection and create a URL.
    dotenv::dotenv()?;
    let listen_addr = dotenv::var("LISTEN_ADDR")?;
    let listener = TcpListener::bind(listen_addr).await?;
    let addr = format!("http://{}", listener.local_addr()?);
    println!("listening on {}", addr);

    // For each incoming TCP connection, spawn a task and call `accept`.
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        task::spawn(async {
            if let Err(err) = accept(stream).await {
                eprintln!("{}", err);
            }
        });
    }
    Ok(())
}

// Take a TCP stream, and convert it into sequential HTTP request / response pairs.
async fn accept(stream: TcpStream) -> http_types::Result<()> {
    println!("starting new connection from {}", stream.peer_addr()?);
    async_h1::accept(stream.clone(), |req| async move {
        let path = req.url().path();
        println!("{} {:?}", req.method(), path);
        if let Some(data) = web::artifact(path.strip_prefix("/").unwrap()) {
            let mut res = Response::new(StatusCode::Ok);
            if path.ends_with(".js") {
                res.insert_header("Content-Type", "application/javascript");
            } else if path.ends_with(".wasm") {
                res.insert_header("Content-Type", "application/wasm");
            }
            res.set_body(data);
            return Ok(res);
        }
        let mut res = Response::new(StatusCode::Ok);
        res.insert_header("Content-Type", "text/html");
        res.set_body(web::INDEX_HTML);
        Ok(res)
    })
    .await?;
    Ok(())
}
