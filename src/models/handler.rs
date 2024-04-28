//! # Oblivion Default Handler
use crate::types::server;

use super::{render::BaseResponse, session::Session};
use oblivion_codegen::async_route;

/// Not Found Handler
///
/// Handling a non-existent route request.
#[async_route]
pub fn not_found(sess: Session) -> server::Result {
    let olps = sess.request.get_ip();

    Ok(BaseResponse::TextResponse(
        format!("Path {} is not found, error with code 404.", olps),
        404,
    ))
}
