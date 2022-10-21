use anyhow::{anyhow, Error, Result};
use hyper::{header, http::request::Parts, Body, Request, StatusCode};
use routerify::{ext::RequestExt, Middleware, Router, RouterService};

use arhiv_core::{entities::BLOBId, prime_server::respond_with_blob};
use rs_utils::http_server::{error_handler, logger_middleware, not_found_handler, ServerResponse};

use super::{
    app::{App, AppResponse},
    public_assets_handler::public_assets_handler,
};

pub fn build_router_service(app: App) -> Result<RouterService<Body, Error>> {
    let router = Router::builder()
        .data(app)
        .middleware(Middleware::post_with_info(logger_middleware))
        //
        .get("/public/:fileName", public_assets_handler)
        //
        .get("/workspace", workspace_page)
        .post("/workspace_api", workspace_api_handler)
        //
        .get("/blobs/:blob_id", blob_handler)
        //
        .any(not_found_handler)
        .err_handler_with_info(error_handler)
        //
        .build()
        .map_err(|err| anyhow!("failed to build router: {}", err))?;

    let service = RouterService::new(router)
        .map_err(|err| anyhow!("failed to build router service: {}", err))?;

    Ok(service)
}

async fn workspace_page(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();

    let response = app.workspace_page()?;

    app.render(response)
}

async fn workspace_api_handler(req: Request<Body>) -> ServerResponse {
    let (parts, body): (Parts, Body) = req.into_parts();

    let app: &App = parts.data().unwrap();

    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .map(|value| value.to_str())
        .transpose()?
        .unwrap_or_default();

    let response = if content_type == "application/json" {
        let body = hyper::body::to_bytes(body).await?;

        app.workspace_api_handler(&body).await?
    } else {
        AppResponse::Status {
            status: StatusCode::UNSUPPORTED_MEDIA_TYPE,
        }
    };

    app.render(response)
}

async fn blob_handler(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();

    let blob_id = req.param("blob_id").unwrap().as_str();
    let blob_id = BLOBId::from_string(blob_id);

    respond_with_blob(&app.arhiv, &blob_id, req.headers()).await
}
