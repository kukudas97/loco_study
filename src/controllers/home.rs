#![allow(clippy::unused_async)]
use axum::response::Html;
use loco_rs::prelude::*;

#[debug_handler]
pub async fn index() -> Html<&'static str> {
    Html(include_str!("../../assets/search.html"))
}

pub fn routes() -> Routes {
    Routes::new().add("/", get(index))
}
