use super::models::Request;

pub struct Session;

impl Session {
    pub fn new() -> Self {
        Self
    }

    pub fn request(&self, method: &str, olps: &str) -> String {
        // 创建请求
        let mut req = Request::new(method, olps);
        let _ = req.prepare();
        self.send(&mut req)
    }

    pub fn send(&self, request: &mut Request) -> String {
        if request.is_prepared() != true {
            let _ = request.prepare();
        }

        // 发送请求
        request.send();
        request.recv().unwrap()
    }

    pub fn get(&mut self, olps: &str) -> String{
        self.request("GET", olps)
    }

    pub fn post(&mut self, olps: &str) -> String{
        self.request("POST", olps)
    }
}
