use futures::future::BoxFuture;

pub use crate::models::client::Response;
pub type EncryptedData = (Vec<u8>, Vec<u8>, Vec<u8>);
pub type Callback = std::sync::Arc<
    fn(Response, std::sync::Arc<crate::models::session::Session>) -> BoxFuture<'static, bool>,
>;
pub use crate::models::render::BaseResponse;
pub type ServerResponse = BoxFuture<'static, anyhow::Result<BaseResponse>>;
pub type Handler = fn(crate::models::session::Session) -> ServerResponse;
