use crate::utils::parser::OblivionRequest;

#[derive(Clone, Debug, PartialEq)]
pub enum OblivionException {
    ErrorNotPrepared(Option<String>),
    BadProtocol(Option<String>),
    ConnectionRefusedError(Option<String>),
    InvalidOblivion(Option<String>),
    AddressAlreadyInUse(Option<String>),
    UnexpectedDisconnection(Option<String>),
    BadBytes(Option<String>),
    ConnectTimedOut(Option<String>),
    DataTooLarge(Option<String>),
    AllAttemptsRetryFailed(Option<String>),
    UnsupportedMethod(Option<String>),
    ServerError(Option<OblivionRequest>, i32),
}

impl OblivionException {
    fn write_error(
        f: &mut core::fmt::Formatter,
        name: &str,
        info: &Option<String>,
    ) -> core::fmt::Result {
        if info.is_none() {
            let info = "Unknown";
            f.write_str(&format!("oblivion::exceptions::{}: {}", name, info))
        } else {
            let info = info.clone().unwrap();
            f.write_str(&format!("oblivion::exceptions::{}: {}", name, info))
        }
    }
}

impl core::fmt::Display for OblivionException {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::ErrorNotPrepared(info) => {
                OblivionException::write_error(f, "ErrorNotPrepared", info)
            }
            Self::BadProtocol(info) => OblivionException::write_error(f, "BadProtocol", info),
            Self::ConnectionRefusedError(info) => {
                OblivionException::write_error(f, "ConnectionRefusedError", info)
            }
            Self::InvalidOblivion(info) => {
                OblivionException::write_error(f, "InvalidOblivion", info)
            }
            Self::AddressAlreadyInUse(info) => {
                OblivionException::write_error(f, "AddressAlreadyInUse", info)
            }
            Self::UnexpectedDisconnection(info) => {
                OblivionException::write_error(f, "UnexceptedDisconnection", info)
            }
            Self::BadBytes(info) => OblivionException::write_error(f, "BadBytes", info),
            Self::ConnectTimedOut(info) => {
                OblivionException::write_error(f, "ConnectTimedOut", info)
            }
            Self::DataTooLarge(info) => OblivionException::write_error(f, "DataTooLarge", info),
            Self::AllAttemptsRetryFailed(info) => {
                OblivionException::write_error(f, "AllAttemptsRetryFailed", info)
            }
            Self::UnsupportedMethod(info) => {
                OblivionException::write_error(f, "UnsupportedMethod", info)
            }
            Self::ServerError(request, status_code) => {
                let request = request.clone();
                if request.is_none() {
                    OblivionException::write_error(f, "ServerError", &Some(format!("ServerError code {}", status_code)))
                } else {
                    let mut request = request.unwrap();
                    OblivionException::write_error(
                        f,
                        "ServerError",
                        &format!(
                            "{}/{} {} From {} {} {}",
                            request.get_protocol(),
                            request.get_version(),
                            request.get_method(),
                            request.get_ip(),
                            request.get_olps(),
                            status_code
                        )
                        .into(),
                    )
                }
            }
        }
    }
}
