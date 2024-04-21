//! # Oblivion Default Handler
use super::{
    render::{BaseResponse, Response},
    session::Session,
};
use oblivion_codegen::async_route;

/// Not Found Handler
///
/// Handling a non-existent route request.
#[async_route]
pub fn not_found(mut sess: Session) -> Response {
    let olps = sess.request.as_mut().unwrap().get_ip();

    Ok(BaseResponse::TextResponse(
        format!("Path {} is not found, error with code 404.", olps),
        404,
    ))
}
