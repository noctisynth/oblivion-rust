use futures::future::BoxFuture;

use crate::models::render::BaseResponse;
pub type Result = BoxFuture<'static, anyhow::Result<BaseResponse>>;
pub type Response = Result;
