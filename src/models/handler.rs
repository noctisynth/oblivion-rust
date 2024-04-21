//! # Oblivion Default Handler
use super::render::{BaseResponse, Response};
use super::session::Session;
use futures::FutureExt;

/// Not Found Handler
///
/// Handling a non-existent route request.
pub fn not_found(session: &mut Session) -> Response {
    let olps = session.request.as_mut().unwrap().get_ip();
    async move {
        Ok(BaseResponse::TextResponse(
            format!("Path {} is not found, error with code 404.", olps),
            404,
        ))
    }
    .boxed()
}
