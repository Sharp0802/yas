#![feature(coroutines)]
#![feature(gen_blocks)]
#![feature(async_iterator)]
#![feature(str_as_str)]
#![feature(associated_type_defaults)]

mod tools;
mod defs;
mod chat;

use crate::chat::{add_chat, process_chat};
use crate::defs::*;
use crate::tools::search_fs_decl;
use bytes::Bytes;
use dotenv::dotenv;
use google_ai_rs::{Client, GenerativeModel, Tool};
use http::{header, Method, Request, Response, StatusCode};
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::convert::Infallible;
use std::env::var_os;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::OnceLock;
use tokio::net::TcpListener;
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;

type ResponseResult = Result<Response<BoxBody<Bytes, Infallible>>, Box<dyn Error + Send + Sync>>;

static CLIENT: OnceLock<Client> = OnceLock::new();
static MODEL: OnceLock<GenerativeModel> = OnceLock::new();

async fn get_chat() -> ResponseResult {
    let chat = chat::get_chat().await;
    let json = serde_json::to_string(&chat)?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Full::from(Bytes::from(json)).boxed())
        .unwrap())
}

async fn post_chat(req: Request<Incoming>) -> ResponseResult {
    let body = req.collect().await?.to_bytes();
    let chat = match serde_json::from_slice::<Content>(&body) {
        Ok(chat) => chat,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from(e.to_string())).boxed())?);
        }
    };

    let (sender, receiver) = channel(256);

    tokio::spawn(async move {
        add_chat(chat).await;
        process_chat(sender).await;
    });

    let stream = ReceiverStream::new(receiver);
    let stream_body = StreamBody::new(stream);

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(stream_body.boxed())?)
}

macro_rules! static_file {
    ($name:expr, $mime:expr) => {
        ($name, ($mime, Bytes::from_static(include_bytes!(concat!("www", $name)))))
    };
}

async fn handle_request(req: Request<Incoming>) -> ResponseResult {

    let files: HashMap<&'static str, (&'static str, Bytes)> = HashMap::from([
        static_file!("/index.html", "text/html"),
        static_file!("/main.js", "text/javascript"),
        static_file!("/sse.js", "text/javascript"),
        static_file!("/style.css", "text/css"),
    ]);

    let path = match req.uri().path() {
        "/" => "/index.html",
        v => v
    };

    match (req.method(), path) {
        (&Method::GET, "/chat") => get_chat().await,
        (&Method::POST, "/chat") => post_chat(req).await,

        (&Method::GET, p) => {
            let Some((mime, b)) = files.get(p) else {
                return Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::new()).boxed())?)
            };

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.to_string())
                .body(Full::new(b.clone()).boxed())?)
        }

        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from_static(b"Not Found")).boxed())?),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let Some(api_key) = var_os("GEMINI_API_KEY") else {
        panic!("variable GEMINI_API_KEY not set");
    };
    let Some(api_key) = api_key.to_str() else {
        panic!("variable GEMINI_API_KEY has invalid characters");
    };

    let client = Client::new(api_key.into()).await?;
    CLIENT.set(client).unwrap();

    let mut model = GenerativeModel::new(CLIENT.get().unwrap(), "gemini-2.5-pro");

    model.tools = Some(vec![Tool {
        function_declarations: vec![search_fs_decl()],
        ..Tool::default()
    }]);

    MODEL.set(model).unwrap();

    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle_request))
                .await
            {
                eprintln!("error serving connection: {:?}", err);
            }
        });
    }
}
