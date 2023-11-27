use oblivion::api::get;

#[tokio::main]
async fn main() {
    let mut r = get("oblivion://127.0.0.1:813/test", true).await.unwrap();
    println!("{}", r.text())
}
