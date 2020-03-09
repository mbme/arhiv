use super::Prime;
use crate::entities::*;
use bytes::Bytes;
use std::sync::{Arc, Mutex};
use warp::{http, Filter};

impl Prime {
    #[tokio::main]
    pub async fn start_server(self) {
        let port = self.config.port;
        let prime = Arc::new(Mutex::new(self));

        // POST /attachment-data/:id
        let post_attachment_data = {
            let prime = prime.clone();

            warp::post()
                .and(warp::path("attachment-data"))
                .and(warp::path::param::<String>())
                .and(warp::body::bytes())
                .map(move |id: String, data: Bytes| {
                    prime
                        .lock()
                        .unwrap()
                        .storage
                        .write_attachment_data(&id, &data)
                        .unwrap();

                    warp::reply::reply()
                })
        };

        // GET /attachment-data/:id
        let get_attachment_data = {
            let prime = prime.clone();

            warp::get()
                .and(warp::path("attachment-data"))
                .and(warp::path::param::<String>())
                .map(move |id: String| {
                    let file = prime.lock().unwrap().storage.get_attachment_data_path(&id);

                    // FIXME stream file, support ranges
                    http::Response::builder().body(std::fs::read(file).unwrap())
                })
        };

        // POST /changeset Changeset
        let post_changeset = {
            let prime = prime.clone();

            warp::post()
                .and(warp::path("changeset"))
                .and(warp::body::json())
                .map(move |changeset: Changeset| {
                    let result = prime.lock().unwrap().exchange(changeset).unwrap();

                    warp::reply::json(&result)
                })
        };

        let routes = post_attachment_data
            .or(get_attachment_data)
            .or(post_changeset);

        warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    }
}
