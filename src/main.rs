
use axum::{
    body::Body, extract::{DefaultBodyLimit, Query}, response::{IntoResponse, Response}, routing::{get, post}, Json, Router
};
use axum_extra::extract::Form;

use rustc_version_runtime::version;
use serde::{Deserialize, Serialize};
use tower_http::{limit::RequestBodyLimitLayer, services::ServeFile};
use html_escape::encode_text;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        //.route_service("/", get(|| async { axum::Redirect::temporary("https://www.regexplanet.com/advanced/rust/index.html") }))
        .route_service("/", get(root_handler))
        .route_service("/test.json", post(test_handler))
        .route_service("/favicon.ico", ServeFile::new("static/favicon.ico"))
        .route_service("/favicon.svg", ServeFile::new("static/favicon.svg"))
        .route_service("/robots.txt", ServeFile::new("static/robots.txt"))
        .route_service("/status.json", get(get_status))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024 /* 10mb */));

    // get address from environment variable
    let address = std::env::var("ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());

    // get port from environment variable
    let port = std::env::var("PORT").unwrap_or_else(|_| "5000".to_string());

    let listen = format!("{}:{}", address, port);

    println!("INFO: Listening on {}", listen);

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
    version: String,
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
        version: format!("{}", version()),
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
        .body(Body::from(format!("Running Rust {}", version())))
        .unwrap();
}

#[derive(Deserialize, Serialize, Debug)]
struct TestInput {
    regex: String,
    #[serde(default)]
    replacement: String,
    #[serde(default)]
    callback: String,
    #[serde(default)]
    options: Vec<String>,
    #[serde(default)]
    #[serde(rename(deserialize = "input"))]
    inputs: Vec<String>,
}

#[derive(Serialize)]
struct TestOutput {
    success: bool,
    message: String,
    html: String,
}

async fn test_handler(Form(test_input): Form<TestInput>) -> Response<Body> {
    
    let mut html = String::new();

    html.push_str("<table class=\"table table-bordered table-striped\" style=\"width: auto;\">\n");
    html.push_str("\t<tbody>\n");
    html.push_str("\t\t<tr>\n");
    html.push_str("\t\t\t<td>Regular Expression</td>\n");
    html.push_str(&format!("\t\t\t<td><code>{}</code></td>\n", encode_text(&test_input.regex)));
    html.push_str("\t\t</tr>\n");

    if test_input.replacement != "" {
        html.push_str("\t\t<tr>\n");
        html.push_str("\t\t\t<td>Replacement</td>\n");
        html.push_str(&format!("\t\t\t<td><code>{}</code></td>\n", encode_text(&test_input.replacement)));
        html.push_str("\t\t</tr>\n");
    }

    html.push_str("\t</tbody>\n");
    html.push_str("</table>");

    let the_regex = regex::RegexBuilder::new(&test_input.regex)
        .case_insensitive(test_input.options.contains(&"i".to_string()))
        .multi_line(test_input.options.contains(&"m".to_string()))
        .dot_matches_new_line(test_input.options.contains(&"s".to_string()))
        .build();

    if the_regex.is_err() {
        let err_msg = the_regex.unwrap_err().to_string();

        html.push_str("<div class=\"alert alert-danger\" role=\"alert\">\n");   
        html.push_str(&format!("<strong>Error:</strong> {}<br>\n", encode_text(&err_msg)));
        html.push_str("</div>\n");

        return handle_jsonp(&test_input.callback, html);
    } 

    if test_input.inputs.len() == 0 {
        html.push_str("<div class=\"alert alert-danger\" role=\"alert\">\n");   
        html.push_str("No inputs to test!\n");
        html.push_str("</div>\n");

        return handle_jsonp(&test_input.callback, html);
    }

    let the_regex = the_regex.unwrap();

    html.push_str("<table class=\"table table-bordered table-striped\" style=\"width: auto;\">\n");
    html.push_str("\t<thead>\n");
    html.push_str("\t\t<tr>\n");
    html.push_str("\t\t\t<th>Input</th>\n");
    html.push_str("\t\t\t<th>is_match</th>\n");
    html.push_str("\t\t</tr>\n");
    html.push_str("\t</thead>\n");
    html.push_str("\t<tbody>\n");

    for input in test_input.inputs {
        //let output = the_regex.as_ref().unwrap().replace_all(&input, &test_input.replacement);
        html.push_str("\t\t<tr>\n");
        html.push_str(&format!("\t\t\t<td>{}</td>\n", encode_text(&input)));
        let is_match = if the_regex.is_match(&input) { "true" } else { "false" };
        html.push_str(&format!("\t\t\t<td>{}</td>\n", is_match));
        html.push_str("\t\t</tr>\n");
    }

    html.push_str("\t</tbody>\n");
    html.push_str("</table>");

    return handle_jsonp(&test_input.callback, html);
}

fn handle_jsonp(callback: &str, html: String) -> Response<Body> {


    let test_output = TestOutput {
        success: true,
        message: "OK".to_string(),
        html: html,
    };

    let json_output = serde_json::to_string(&test_output).unwrap();

    if callback == "" {
        return Response::builder()
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "GET")
            .header("Access-Control-Max-Age", "604800")
            .body(Body::from(json_output))
            .unwrap();
    } else {
        let jsonp = format!("{}({})", callback, json_output);
        return Response::builder()
            .header("Content-Type", "text/html; charset=utf-8")
            .body(Body::from(jsonp))
            .unwrap();
    }
}