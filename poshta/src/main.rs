use serde_json::Value;

mod auth;

const BASE_URL: &str = "https://www.googleapis.com/gmail/v1/users/me";

#[tokio::main]
async fn main() {
    env_logger::init();

    let token = auth::auth().await;

    log::info!("TOKEN {:?}", token);

    let resp = reqwest::Client::new()
        .get(&format!("{}/profile", BASE_URL))
        .bearer_auth(token.as_str())
        .send()
        .await
        .unwrap();

    println!("{:#?}", resp);
    println!("{:?}", resp.json::<Value>().await.unwrap());
}
