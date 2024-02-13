//! # Oblivion Default Handler
use super::render::BaseResponse;
use crate::utils::parser::OblivionRequest;
use futures::future::{BoxFuture, FutureExt};
use oblivion_codegen::async_route;

/// Not Found Handler
///
/// Handling a non-existent route request.
#[async_route]
pub fn not_found(mut request: OblivionRequest) -> BaseResponse {
    BaseResponse::TextResponse(
        format!(
            "Path {} is not found, error with code 404.",
            request.get_olps()
        ),
        404,
    )
}
