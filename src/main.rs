#![feature(trim_prefix_suffix)]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod action;
mod actions;
mod cache;
mod frames;
mod parser;

use crate::{
    action::{ActionList, parse_actions},
    cache::Cache,
    frames::{Frames, get_error_image},
};

use axum::{
    Router,
    extract::{Path},
    http::{self},
    response::{AppendHeaders, IntoResponse},
    routing::get,
};


#[tokio::main]
async fn main() {
    Cache::init().await.unwrap();
    let app = Router::new().route("/pfp/{*path}", get(pfp));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:62673")
        .await
        .unwrap();
    let _ = axum::serve(listener, app).await;
}

async fn pfp(Path(path): Path<String>) -> impl IntoResponse {
    let mut images = Vec::new();
    let action_strings = split_actions(path.trim_suffix(".webp"));
    let mut actions = parse_actions(&action_strings);

    let result = actions.apply_actions(&mut images).await;

    let response_bytes = match result {
        Ok(()) => match images.encode() {
            Ok(bytes) => bytes,
            Err(err) => get_error_image(err.to_string())
                .unwrap_or_else(|e| format!("AAAAAAAAa {e}!").into_bytes()),
        },
        Err(err) => get_error_image(err.to_string())
            .unwrap_or_else(|e| format!("AAAAAAAAa {e}!").into_bytes()),
    };

    (
        http::StatusCode::OK,
        AppendHeaders([(http::header::CONTENT_TYPE, "image/webp")]),
        response_bytes,
    )
        .into_response()
}

fn split_actions(path: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, char) in path.char_indices() {
        match char {
            '(' => depth += 1,
            ')' => depth -= 1,
            '/' if depth == 0 => {
                result.push(&path[start..i]);
                start = i + 1;
            }
            _ => (),
        }
    }

    if start <= path.len() {
        result.push(&path[start..]);
    }

    result
}
