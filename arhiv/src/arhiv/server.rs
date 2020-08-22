use super::Arhiv;
use crate::entities::*;
use crate::storage::Queries;
use anyhow::*;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use warp::{http, reply, Filter};

pub struct Server {
    join_handle: JoinHandle<()>,
    shutdown_sender: oneshot::Sender<()>,
}

impl Server {
    pub fn new(join_handle: JoinHandle<()>, shutdown_sender: oneshot::Sender<()>) -> Self {
        Server {
            join_handle,
            shutdown_sender,
        }
    }

    pub async fn join(self, shutdown: bool) {
        if shutdown {
            self.shutdown_sender
                .send(())
                .expect("must be able to send shutdown signal");
        }

        self.join_handle
            .await
            .expect("must be able to await the server");
    }
}

impl Arhiv {
    pub fn start_server(self) -> Server {
        let port = self.config.server_port;

        let arhiv = Arc::new(self);

        // POST /attachment-data/:id
        let post_attachment_data = {
            let arhiv = arhiv.clone();

            warp::post()
                .and(warp::path("attachment-data"))
                .and(warp::path::param::<String>())
                .and(warp::body::bytes())
                .map(move |id: String, data: _| {
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
                    let result = arhiv.exchange(changeset);

                    match result {
                        Ok(changeset_response) => {
                            reply::with_status(changeset_response.serialize(), http::StatusCode::OK)
                        }
                        err => {
                            log::error!("Failed to apply a changeset: {:?}", err);

                            reply::with_status(
                                format!("failed to apply a changeset: {:?}", err),
                                http::StatusCode::INTERNAL_SERVER_ERROR,
                            )
                        }
                    }
                })
        };

        let routes = post_attachment_data
            .or(get_attachment_data)
            .or(post_changeset);

        let (shutdown_sender, shutdown_receiver) = oneshot::channel();

        let (addr, server) =
            warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], port), async {
                shutdown_receiver.await.ok();
            });

        // Spawn the server into a runtime
        let join_handle = tokio::task::spawn(server);

        log::info!("started server on {}", addr);

        Server::new(join_handle, shutdown_sender)
    }

    fn exchange(&self, changeset: Changeset) -> Result<ChangesetResponse> {
        if !self.config.is_prime {
            return Err(anyhow!("can't exchange: not a prime"));
        }

        if !changeset.is_empty() && self.storage.get_connection()?.has_staged_changes()? {
            return Err(anyhow!("can't exchange: there are staged changes"));
        }

        let base_rev = changeset.base_rev.clone();

        self.apply_changeset(changeset)?;

        self.generate_changeset_response(base_rev)
    }
}
