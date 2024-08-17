
use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Query},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};

use rustc_version_runtime::version;
use serde::{Deserialize, Serialize};
use tower_http::{limit::RequestBodyLimitLayer, services::ServeFile};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        //.route_service("/", get(|| async { Redirect::temporary("https://www.regexplanet.com/advanced/rust/index.html") }))
        .route_service("/", get(root_handler))
        .route_service("/favicon.ico", ServeFile::new("static/favicon.ico"))
        .route_service("/favicon.svg", ServeFile::new("static/favicon.svg"))
        .route_service("/robots.txt", ServeFile::new("static/robots.txt"))
        .route_service("/status.json", get(get_status))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024 /* 10mb */));

    // run our app with hyper, listening globally on port 3000

    // get address from environment variable
    let address = std::env::var("ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());

    // get port from environment variable
    let port = std::env::var("PORT").unwrap_or_else(|_| "4000".to_string());

    let listen = format!("{}:{}", address, port);

    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct StatusParams {
    callback: Option<String>,
}

#[derive(Serialize)]
struct StatusInfo {
    success: bool,
    message: String,
    tech: String,
    timestamp: String,
    lastmod: String,
    commit: String,
}

async fn get_status(Query(params): Query<StatusParams>) -> Response {
    let tech = format!("Rust {}", version());
    let timestamp = chrono::Utc::now().to_rfc3339();
    let lastmod = std::env::var("LASTMOD").unwrap_or_else(|_| "(local)".to_string());
    let commit = std::env::var("COMMIT").unwrap_or_else(|_| "(local)".to_string());

    let status = StatusInfo {
        success: true,
        message: "OK".to_string(),
        tech: tech.to_string(),
        timestamp: timestamp.to_string(),
        lastmod: lastmod.to_string(),
        commit: commit.to_string(),
    };

    if params.callback.is_some() {
        let jsonp = format!(
            "{}({})",
            params.callback.unwrap(),
            serde_json::to_string(&status).unwrap()
        );
        return jsonp.into_response();
    }
    let mut res = Json(status).into_response();
    res.headers_mut().insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    res.headers_mut().insert("Access-Control-Allow-Methods", "GET".parse().unwrap());
    res.headers_mut().insert("Access-Control-Max-Age", "604800".parse().unwrap());
    return res;
}

async fn root_handler() -> Response<Body> {
    return Response::builder()
        .header("Content-Type", "text/plain; charset=utf-8")
        .body(Body::from("Dev server running!"))
        .unwrap();
}