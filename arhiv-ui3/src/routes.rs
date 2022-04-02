use anyhow::{anyhow, Error, Result};
use hyper::{http::request::Parts, Body, Request};
use routerify::{ext::RequestExt, Middleware, Router, RouterService};

use arhiv_core::{
    entities::{BLOBId, Id},
    prime_server::respond_with_blob,
};
use rs_utils::http_server::{
    error_handler, logger_middleware, not_found_handler, parse_urlencoded, RequestQueryExt,
    ServerResponse,
};

use crate::{app::App, public_assets_handler::public_assets_handler, utils::extract_fields};

pub fn build_router_service(app: App) -> Result<RouterService<Body, Error>> {
    let router = Router::builder()
        .data(app)
        .middleware(Middleware::post_with_info(logger_middleware))
        .get("/public/:fileName", public_assets_handler)
        .get("/", index_page)
        //
        .get("/new", new_document_variants_page)
        .get("/new/:document_type", new_document_page)
        .post("/new/:document_type", new_document_page_handler)
        .get(
            "/collections/:collection_id/new/:document_type",
            new_document_page,
        )
        .post(
            "/collections/:collection_id/new/:document_type",
            new_document_page_handler,
        )
        //
        .get("/catalogs/:document_type", catalog_page)
        .get("/erased", erased_documents_list_page)
        //
        .get("/documents/:id", document_page)
        .get("/collections/:collection_id/documents/:id", document_page)
        //
        .get("/documents/:id/edit", edit_document_page)
        .post("/documents/:id/edit", edit_document_page_handler)
        .get("/documents/:id/erase", erase_document_confirmation_dialog)
        .post(
            "/documents/:id/erase",
            erase_document_confirmation_dialog_handler,
        )
        //
        .get("/collections/:collection_id/documents/:id", document_page)
        .get(
            "/collections/:collection_id/documents/:id/edit",
            edit_document_page,
        )
        .post(
            "/collections/:collection_id/documents/:id/edit",
            edit_document_page_handler,
        )
        .get(
            "/collections/:collection_id/documents/:id/erase",
            erase_document_confirmation_dialog,
        )
        .post(
            "/collections/:collection_id/documents/:id/erase",
            erase_document_confirmation_dialog_handler,
        )
        //
        .get("/blobs/:blob_id", blob_handler)
        //
        .get("/modals/pick-document", pick_document_modal)
        .get("/modals/pick-file", pick_file_modal)
        .get(
            "/modals/pick-file-confirmation",
            pick_file_confirmation_modal,
        )
        .post(
            "/modals/pick-file-confirmation",
            pick_file_confirmation_modal_handler,
        )
        //
        .get("/modals/scrape", scrape_modal)
        .post("/modals/scrape", scrape_modal_handler)
        //
        .get("/apps/player", app_player)
        //
        .get("/api/documents/:id", api_get_document)
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

async fn index_page(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();

    let response = app.index_page()?;

    app.render(response)
}

async fn new_document_variants_page(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();

    let response = app.new_document_variants_page()?;

    app.render(response)
}

async fn new_document_page(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();

    let document_type = req
        .param("document_type")
        .expect("document_type must be present");

    let parent_collection: Option<Id> = req.param("collection_id").map(Into::into);

    let response = app.new_document_page(document_type, &parent_collection)?;

    app.render(response)
}

async fn new_document_page_handler(req: Request<Body>) -> ServerResponse {
    let (parts, body): (Parts, Body) = req.into_parts();
    let app: &App = parts.data().unwrap();

    let document_type = parts
        .param("document_type")
        .expect("document_type must be present");

    let parent_collection: Option<Id> = parts.param("collection_id").map(Into::into);

    let fields = extract_fields(body).await?;

    let response = app.new_document_page_handler(document_type, &parent_collection, &fields)?;

    app.render(response)
}

async fn catalog_page(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();
    let document_type: &String = req.param("document_type").unwrap();

    let url = req.get_url();

    let response = app.catalog_page(document_type, url)?;

    app.render(response)
}

async fn erased_documents_list_page(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();

    let response = app.erased_documents_list_page()?;

    app.render(response)
}

async fn document_page(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();

    let id: Id = req.param("id").unwrap().into();
    let parent_collection: Option<Id> = req.param("collection_id").map(Into::into);
    let url = req.get_url();

    let response = app.document_page(&id, &parent_collection, url)?;

    app.render(response)
}

async fn edit_document_page(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();

    let id: Id = req.param("id").unwrap().into();
    let parent_collection: Option<Id> = req.param("collection_id").map(Into::into);

    let response = app.edit_document_page(&id, &parent_collection)?;

    app.render(response)
}

async fn edit_document_page_handler(req: Request<Body>) -> ServerResponse {
    let (parts, body): (Parts, Body) = req.into_parts();
    let app: &App = parts.data().unwrap();

    let id: Id = parts.param("id").unwrap().into();
    let parent_collection: Option<Id> = parts.param("collection_id").map(Into::into);

    let fields = extract_fields(body).await?;

    let response = app.edit_document_page_handler(&id, &parent_collection, &fields)?;

    app.render(response)
}

async fn erase_document_confirmation_dialog(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();

    let id: Id = req.param("id").unwrap().into();
    let parent_collection: Option<Id> = req.param("collection_id").map(Into::into);

    let response = app.erase_document_confirmation_dialog(&id, &parent_collection)?;

    app.render(response)
}

async fn erase_document_confirmation_dialog_handler(req: Request<Body>) -> ServerResponse {
    let (parts, body): (Parts, Body) = req.into_parts();
    let app: &App = parts.data().unwrap();

    let id: Id = parts.param("id").unwrap().into();
    let parent_collection: Option<Id> = parts.param("collection_id").map(Into::into);

    let body = hyper::body::to_bytes(body).await?;
    let fields = parse_urlencoded(&body);

    let response =
        app.erase_document_confirmation_dialog_handler(&id, &parent_collection, &fields)?;

    app.render(response)
}

async fn blob_handler(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();

    let blob_id = req.param("blob_id").unwrap().as_str();
    let blob_id = BLOBId::from_string(blob_id);

    respond_with_blob(&app.arhiv, &blob_id, req.headers()).await
}

async fn pick_document_modal(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();
    let url = req.get_url();

    let response = app.pick_document_modal(url)?;

    app.render(response)
}

async fn pick_file_modal(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();
    let url = req.get_url();

    let response = App::pick_file_modal(url)?;

    app.render(response)
}

async fn pick_file_confirmation_modal(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();

    let url = req.get_url();

    let response = App::pick_file_confirmation_modal(&url)?;

    app.render(response)
}

async fn pick_file_confirmation_modal_handler(req: Request<Body>) -> ServerResponse {
    let (parts, body): (Parts, Body) = req.into_parts();
    let app: &App = parts.data().unwrap();
    let fields = extract_fields(body).await?;

    let response = app.pick_file_confirmation_modal_handler(fields).await?;

    app.render(response)
}

async fn scrape_modal(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();

    let response = App::scrape_modal()?;

    app.render(response)
}

async fn scrape_modal_handler(req: Request<Body>) -> ServerResponse {
    let (parts, body): (Parts, Body) = req.into_parts();
    let app: &App = parts.data().unwrap();

    let body = hyper::body::to_bytes(body).await?;
    let fields = parse_urlencoded(&body);

    let response = app.scrape_modal_handler(&fields).await?;

    app.render(response)
}

async fn app_player(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();

    let response = app.player_app_page()?;

    app.render(response)
}

async fn api_get_document(req: Request<Body>) -> ServerResponse {
    let app: &App = req.data().unwrap();

    let id: Id = req.param("id").unwrap().into();

    let response = app.document_api(&id)?;

    app.render(response)
}
