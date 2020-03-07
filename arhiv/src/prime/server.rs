use super::Prime;
use warp::Filter;

impl Prime {
    #[tokio::main]
    pub async fn start_server(self) {
        // GET /hello/warp => 200 OK with body "Hello, warp!"
        let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

        warp::serve(hello)
            .run(([127, 0, 0, 1], self.config.port))
            .await;
    }
}
