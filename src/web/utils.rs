use actix_web::{HttpResponse, ResponseError};
use serde_json::json;

use crate::AiterError;

impl ResponseError for AiterError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::Ok().json(json!({
            "error": "SERVER_ERROR",
            "message": self.to_string(),
        }))
    }
}
