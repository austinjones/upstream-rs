use block_id::{Alphabet, BlockId};
use clap::Parser;

use colored::Colorize;
use hyper::service::{make_service_fn, service_fn};
use hyper::{body, Body, Request, Response, Server};
use rand::{thread_rng, Rng};
use serde::Serialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::OnceLock;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to bind
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();
    let args = Args::parse();
    // This address is localhost

    let addr: SocketAddr = ([127, 0, 0, 1], args.port).into();

    let make_svc = make_service_fn(move |_conn| async { Ok::<_, Infallible>(service_fn(serve)) });
    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

async fn serve(mut req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let id = generate_id();

    let mut msg = "".to_string();
    let mut header = "".to_string();
    header += &id;
    header += " ";
    header += req.method().as_str();
    header += " ";
    header += req.uri().path();
    msg += &format!("{}", header.bold().blue());
    msg += "\n";

    for (name, value) in req.headers().into_iter() {
        msg += &format!("{}", name.as_str().bold());
        msg += ": ";
        msg += value.to_str().unwrap_or("<non-unicode>");
        msg += "\n";
    }

    if let Ok(bytes) = body::to_bytes(req.body_mut()).await {
        let bytes: &[u8] = &*bytes;

        let body = if let Ok(json) = serde_json::from_slice::<serde_json::Value>(bytes) {
            serde_json::to_string_pretty(&json).unwrap_or_else(|_e| "<encoding error>".to_string())
        } else {
            String::from_utf8_lossy(bytes).to_string()
        };

        msg += &format!("{}", body.red());
    } else {
        msg += "<body error>";
    }
    msg += "\n\n";
    println!("{}", msg);

    let response_data = ResponseJson { id };
    let response_json = serde_json::to_string_pretty(&response_data).expect("should encode");
    Ok(Response::new(Body::from(response_json)))
}

fn generate_id() -> String {
    static BLOCK_ID: OnceLock<BlockId<char>> = OnceLock::new();
    let block_id = BLOCK_ID.get_or_init(|| {
        let alphabet = Alphabet::alphanumeric();
        BlockId::new(alphabet, thread_rng().gen(), 4)
    });

    let id = block_id
        .encode(thread_rng().gen())
        .expect("Should generate id");
    let mut string = String::with_capacity(id.len());

    for char in id {
        string.push(char);
    }

    string
}

#[derive(Serialize)]
struct ResponseJson {
    pub id: String,
}
