use super::Arhiv;
use crate::entities::*;
use bytes::Bytes;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use warp::{http, reply, Filter};

impl Arhiv {
    #[tokio::main]
    pub async fn start_server(self) {
        let port = self.config.server_port;

        let arhiv = Arc::new(Mutex::new(self));

        // POST /attachment-data/:id
        let post_attachment_data = {
            let arhiv = arhiv.clone();

            warp::post()
                .and(warp::path("attachment-data"))
                .and(warp::path::param::<String>())
                .and(warp::body::bytes())
                .map(move |id: String, data: Bytes| {
                    let arhiv = arhiv.lock().unwrap();
                    let dst = arhiv.storage.get_staged_attachment_file_path(&id);

                    if Path::new(&dst).exists() {
                        log::error!("temp attachment data {} already exists", dst);

                        // FIXME check hashes instead of throwing an error
                        return reply::with_status(
                            format!("temp attachment data {} already exists", dst),
                            http::StatusCode::CONFLICT,
                        );
                    }

                    if let Err(err) = fs::write(dst, &data) {
                        return reply::with_status(
                            format!("failed to write data: {}", err),
                            http::StatusCode::INTERNAL_SERVER_ERROR,
                        );
                    }

                    reply::with_status("".to_string(), http::StatusCode::OK)
                })
        };

        // GET /attachment-data/:id
        let get_attachment_data = {
            let arhiv = arhiv.clone();

            warp::get()
                .and(warp::path("attachment-data"))
                .and(warp::path::param::<String>())
                .map(move |id: String| {
                    let arhiv = arhiv.lock().unwrap();
                    let file = arhiv.storage.get_committed_attachment_file_path(&id);

                    // FIXME stream file, support ranges
                    // res.headers['Content-Disposition'] = `inline; filename=${fileId}`
                    // res.headers['Content-Type'] = await getMimeType(filePath)
                    // res.headers['Cache-Control'] = 'immutable, private, max-age=31536000' // max caching

                    http::Response::builder().body(std::fs::read(file).unwrap())
                })
        };

        // POST /changeset Changeset
        let post_changeset = {
            let arhiv = arhiv.clone();

            warp::post()
                .and(warp::path("changeset"))
                .and(warp::body::json())
                .map(move |changeset: Changeset| {
                    let arhiv = arhiv.lock().unwrap();
                    let mut attachment_data = HashMap::new();

                    for attachment in &changeset.attachments {
                        let id = attachment.id.clone();
                        let path = arhiv.storage.get_staged_attachment_file_path(&id);

                        attachment_data.insert(id, path);
                    }

                    let result = arhiv.exchange(changeset, attachment_data).unwrap();
                    // FIXME preprocess changeset

                    reply::json(&result)
                })
        };

        let routes = post_attachment_data
            .or(get_attachment_data)
            .or(post_changeset);

        warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    }
}
