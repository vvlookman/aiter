use actix_web::*;

use crate::api;

pub mod ai;
pub mod chat;
pub mod doc;
pub mod llm;
pub mod skill;
pub mod tool;

#[get("/version")]
pub async fn version() -> impl Responder {
    api::sys::get_version().await
}
