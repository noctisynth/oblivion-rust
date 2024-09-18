//! # Oblivion Default Handler
use crate::types::server;

use super::{render::BaseResponse, session::Session};
use oblivion_codegen::async_route;

/// Not Found Handler
///
/// Handling a non-existent route request.
#[async_route]
pub fn not_found(session: Session) -> server::Result {
    let entrance = session.request.get_ip();

    Ok(BaseResponse::TextResponse(
        format!("Path {} is not found, error with code 404.", entrance),
        404,
    ))
}
