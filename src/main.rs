#![feature(trim_prefix_suffix)]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod action;
mod actions;
mod cache;
mod discord;
mod frames;
mod twitch;

use std::sync::Arc;

use crate::{
    action::{ActionList, parse_actions},
    cache::Cache,
    discord::DiscordPFP,
    frames::{Frames, get_error_image},
};
use axum::{
    Router,
    extract::{Path, State},
    http::{self},
    response::{AppendHeaders, IntoResponse},
    routing::get,
};

pub struct AppState {
    discord: DiscordPFP,
}

impl AppState {
    fn new() -> Self {
        Self {
            discord: DiscordPFP::build(),
        }
    }
}

#[tokio::main]
async fn main() {
    Cache::init().await.unwrap();
    let state: Arc<AppState> = Arc::new(AppState::new());

    let app = Router::new()
        .route("/pfp/{*path}", get(pfp))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:62673")
        .await
        .unwrap();
    let _ = axum::serve(listener, app).await;
}

async fn pfp(State(state): State<Arc<AppState>>, Path(path): Path<String>) -> impl IntoResponse {
    let mut images = Vec::new();
    let action_strings = split_actions(path.trim_suffix(".webp"));
    let mut actions = parse_actions(&action_strings, &state);

    let result = actions.apply_actions(&mut images).await;

    let response_bytes = match result {
        Ok(()) => match images.encode() {
            Ok(bytes) => bytes,
            Err(err) => {
                get_error_image(err).unwrap_or_else(|e| format!("AAAAAAAAa {e}!").into_bytes())
            }
        },
        Err(err) => get_error_image(err).unwrap_or_else(|e| format!("AAAAAAAAa {e}!").into_bytes()),
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
            '/' => {
                if depth == 0 {
                    result.push(&path[start..i]);
                    start = i + 1;
                }
            }
            _ => (),
        }
    }

    if start <= path.len() {
        result.push(&path[start..]);
    }

    result
}
