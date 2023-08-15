use block_id::{Alphabet, BlockId};
use clap::Parser;

use colored::Colorize;
use hyper::http::{HeaderName, HeaderValue};
use hyper::service::{make_service_fn, service_fn};
use hyper::{body, Body, Request, Response, Server};
use log::info;
use rand::{thread_rng, Rng};
use serde::Serialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};

/// Simple program to greet a person
#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to bind
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    #[arg(short, long)]
    quiet: bool,

    #[arg(short, long)]
    delay_headers: Option<u64>,

    #[arg(short, long)]
    delay_body: Option<u64>,

    #[arg(short, long)]
    size_headers: Option<usize>,

    #[arg(short, long)]
    size_body: Option<usize>,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();
    let args = Args::parse();

    let addr: SocketAddr = ([127, 0, 0, 1], args.port).into();

    let config = Arc::new(args.clone());
    let make_svc = make_service_fn(move |_conn| {
        let config = config.clone();
        async {
            Ok::<_, Infallible>(service_fn(move |req| {
                let config = config.clone();
                serve(req, config)
            }))
        }
    });
    let server = Server::bind(&addr).serve(make_svc);

    info!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

async fn serve(mut req: Request<Body>, config: Arc<Args>) -> Result<Response<Body>, Infallible> {
    let id = generate_id();

    if !config.quiet {
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
                serde_json::to_string_pretty(&json)
                    .unwrap_or_else(|_e| "<encoding error>".to_string())
            } else {
                String::from_utf8_lossy(bytes).to_string()
            };

            msg += &format!("{}", body.red());
        } else {
            msg += "<body error>";
        }
        msg += "\n\n";
        println!("{}", msg);
    }

    let config_for_response = config.clone();
    let mut response = Response::new(Body::wrap_stream(async_stream::stream! {
        crate::maybe_delay!(config_for_response.delay_body);
        if let Some(data_size) = config_for_response.size_body {
            let data: String = random_string(data_size, 39);

            let response_data = ResponseDataJson { id, data };
            let result: Result<_, std::io::Error> = Ok(serde_json::to_string_pretty(&response_data).expect("should encode"));
            yield result;
        } else {
            let response_data = ResponseJson { id };
            let result: Result<_, std::io::Error> = Ok(serde_json::to_string_pretty(&response_data).expect("should encode"));
            yield result;
        }
    }));

    response.headers_mut().insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/json; charset=utf8"),
    );

    if let Some(size) = config.size_headers {
        let value = random_string(size, 146);
        response.headers_mut().insert(
            HeaderName::from_static("x-header-data"),
            HeaderValue::from_str(&value).expect("Header data should encode"),
        );
    }

    crate::maybe_delay!(config.delay_headers);
    Ok(response)
}

#[macro_export]
macro_rules! maybe_delay {
    ($opt: expr) => {
        if let Some(value) = $opt {
            tokio::time::sleep(std::time::Duration::from_millis(value)).await
        }
    };
}

/// Generates a random string, with `offset` characters subtracted.
/// This can be used to set a desired total length for the headers or body
fn random_string(len: usize, offset: usize) -> String {
    if len < offset {
        return "".to_string();
    }
    let len = len - offset;

    use rand::distributions::Alphanumeric;
    let data: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect();
    data
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

#[derive(Serialize)]
struct ResponseDataJson {
    pub id: String,
    pub data: String,
}
