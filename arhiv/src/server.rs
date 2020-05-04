use crate::arhiv::Arhiv;
use crate::entities::*;
use crate::utils::dir_exists;
use bytes::Bytes;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use warp::{http, reply, Filter};

#[tokio::main]
pub async fn start_server(arhiv: Arhiv) {
    let port = arhiv.config.server_port;

    let temp_dir = format!("{}/temp-attachment-data", arhiv.config.arhiv_root);
    if !dir_exists(&temp_dir).unwrap() {
        fs::create_dir(&temp_dir).expect("must be able to create temp-attachment-data dir");
    }

    let get_temp_attachment_file = Arc::new(move |id: &str| format!("{}/{}", &temp_dir, &id));

    let arhiv = Arc::new(Mutex::new(arhiv));

    // POST /attachment-data/:id
    let post_attachment_data = {
        let get_temp_attachment_file = get_temp_attachment_file.clone();

        warp::post()
            .and(warp::path("attachment-data"))
            .and(warp::path::param::<String>())
            .and(warp::body::bytes())
            .map(move |id: String, data: Bytes| {
                let dst = get_temp_attachment_file(&id);
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
                let file = arhiv.lock().unwrap().get_attachment_data_path(&id);

                // FIXME stream file, support ranges
                http::Response::builder().body(std::fs::read(file).unwrap())
            })
    };

    // POST /changeset Changeset
    let post_changeset = {
        let arhiv = arhiv.clone();
        let get_temp_attachment_file = get_temp_attachment_file.clone();

        warp::post()
            .and(warp::path("changeset"))
            .and(warp::body::json())
            .map(move |changeset: Changeset| {
                let mut attachment_data = HashMap::new();

                for attachment in &changeset.attachments {
                    let id = attachment.id.clone();
                    let path = get_temp_attachment_file(&id);

                    attachment_data.insert(id, path);
                }

                let result = arhiv
                    .lock()
                    .unwrap()
                    .exchange(changeset, attachment_data)
                    .unwrap();
                // FIXME preprocess changeset

                reply::json(&result)
            })
    };

    let routes = post_attachment_data
        .or(get_attachment_data)
        .or(post_changeset);

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}
