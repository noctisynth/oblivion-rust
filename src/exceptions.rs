#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ErrorNotPrepared;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BadProtocol;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConnectionRefusedError;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InvalidOblivion;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AddressAlreadyInUse;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnexceptedDisconnection;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BadBytes;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConnectTimedOut;

impl core::fmt::Display for BadProtocol {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("oblivion::exceptions::BadProtocol")
    }
}

impl core::fmt::Display for ErrorNotPrepared {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("oblivion::exceptions::BadProtocol")
    }
}

impl core::fmt::Display for ConnectionRefusedError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("oblivion::exceptions::ConnectionRefusedError")
    }
}

impl core::fmt::Display for InvalidOblivion {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("oblivion::exceptions::InvalidOblivion")
    }
}

impl core::fmt::Display for AddressAlreadyInUse {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("oblivion::exceptions::AdressAlreadyInUse")
    }
}

impl core::fmt::Display for UnexceptedDisconnection {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("oblivion::exceptions::UnexceptedDisconnection")
    }
}

impl core::fmt::Display for BadBytes {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("oblivion::exceptions::BadBytes")
    }
}

impl core::fmt::Display for ConnectTimedOut {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("oblivion::exceptions::ConnectTimedOut")
    }
}
