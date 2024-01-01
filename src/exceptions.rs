//! # Oblivion 异常
//! 所有 Oblivion 函数的异常均返回`OblivionException`。
use scrypt::errors::InvalidOutputLen;
use thiserror::Error;

/// ## Oblivion 异常迭代器
/// 使用迭代器作为函数返回的异常类型。
///
/// 除`ServerError`外，`OblivionException`均需要传入一个`Option<String>`。
#[derive(Error, Debug, Clone, PartialEq)]
pub enum OblivionException {
    #[error("请求尚未预处理")]
    ErrorNotPrepared,
    #[error("错误的协议头: {header}")]
    BadProtocol { header: String },
    #[error("向服务端的链接请求被拒绝, 这可能是由于权限不足或服务端遭到攻击.")]
    ConnectionRefusedError,
    #[error("错误的Oblivion地址: {olps}")]
    InvalidOblivion { olps: String },
    #[error("目标地址[{ipaddr}:{port}]已经被占用.")]
    AddressAlreadyInUse { ipaddr: String, port: i32 },
    #[error("与远程主机的连接被意外断开, 可能是链接被手动切断或遭到了网络审查.")]
    UnexpectedDisconnection,
    #[error("传输的字节流解码失败.")]
    BadBytes,
    #[error("请求被超时, 这可能是由于网络问题或服务端遭到攻击.")]
    ConnectTimedOut,
    #[error("超出预计的数据包大小: {size}")]
    DataTooLarge { size: usize },
    #[error("请求重试失败: {times}")]
    AllAttemptsRetryFailed { times: i32 },
    #[error("方法[{method}]未被支持.")]
    UnsupportedMethod { method: String },
    #[error("Oblivion/1.1 {method} From {ipaddr} {olps} {status_code}")]
    ServerError {
        method: String,
        ipaddr: String,
        olps: String,
        status_code: i32,
    },
    #[error("公钥不合法: {error:?}")]
    PublicKeyInvalid {
        #[from]
        error: elliptic_curve::Error,
    },
    #[error("共享密钥生成时出现异常: {error:?}")]
    InvalidOutputLen { error: InvalidOutputLen },
}
