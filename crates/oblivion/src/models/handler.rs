//! # Oblivion Default Handler
use crate::types::ServerResponse;

use super::{render::BaseResponse, session::Session};
use oblivion_codegen::internal_handler;

/// Not Found Handler
///
/// Handling a non-existent route request.
#[internal_handler]
pub fn not_found(session: Session) -> ServerResponse {
    let entrance = session.request.get_ip();

    Ok(BaseResponse::TextResponse(format!(
        "Path {} is not found, error with code 404.",
        entrance
    )))
}
