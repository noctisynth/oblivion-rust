use super::sessions::Session;

pub(crate) fn request(method: &str, olps: &str) -> String {
    let session = Session::new();
    session.request(method, olps)
}

#[allow(dead_code)]
pub(crate) fn get(olps: &str) -> String {
    request("get", olps)
}

#[allow(dead_code)]
pub(crate) fn post(olps: &str) -> String {
    request("post", olps)
}

#[allow(dead_code)]
pub(crate) fn forward(olps: &str) -> String {
    request("forward", olps)
}