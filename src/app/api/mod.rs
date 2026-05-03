pub mod handler;

use std::{
    env,
    sync::{self, Arc, Mutex},
};

use axum::{
    body::Body,
    http::header,
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use crate::{
    app::{input::context::Context, window::ui},
    config::get_config,
};

pub static PORT: sync::RwLock<Option<u16>> = sync::RwLock::new(None);

pub fn get_api_port() -> Option<u16> {
    PORT.read().ok().and_then(|port| *port)
}

pub async fn listen_and_serve(ctx: Arc<Mutex<Context>>) -> Result<(), std::io::Error> {
    let state = AppState { input_ctx: ctx };

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(
            OpenApiRouter::new()
                .routes(routes!(handler::steam_status::steam_status))
                .routes(routes!(
                    handler::on_overlay_state_changed::on_overlay_state_changed
                ))
                .routes(routes!(handler::steam_cef_reachable::steam_cef_reachable))
                .routes(routes!(handler::get_input_info::get_input_info))
                .routes(routes!(handler::inject_overlay_notifier::inject_overlay_notifier))
                .routes(routes!(handler::connect_viiper::connect_viiper))
                .routes(routes!(handler::show_hide_ui::change_ui_state))
                .routes(routes!(handler::enable_cef_remote_debugging::enable_cef_remote_debug))
                .routes(routes!(handler::create_marker_shortcut::create_marker_shortcut))
                .routes(routes!(handler::shutdown::shutdown))
                .routes(routes!(handler::restart_steam::restart_steam))
                .routes(routes!(handler::restart_sisr::restart_sisr)),
        )
        .split_for_parts();

    if env::var("DEV") == Ok("1".to_string()) {
        let api_doc_yaml = api.to_yaml().expect("Failed to serialize API doc");
        let api_doc_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("openapi.yaml");
        std::fs::write(api_doc_path, api_doc_yaml).expect("Failed to write API doc to disk");
    }

    let router = router
        .with_state(state)
        .merge(Scalar::with_url("/docs", api))
        .fallback(ui::files::static_handler)
        .layer(middleware::from_fn(api_error_middleware))
        .layer(middleware::from_fn(cors_middleware));

    let cfg_port = get_config().port.unwrap_or(0);

    let listener = TcpListener::bind(format!("localhost:{}", cfg_port))
        .await
        .expect("Failed to start API listener");

    if let Ok(mut port) = PORT.write() {
        *port = Some(
            listener
                .local_addr()
                .expect("Failed to read API port")
                .port(),
        );
    }
    tracing::info!(
        "API server listening on port {}",
        get_api_port().unwrap_or(0)
    );

    axum::serve(listener, router).await
}

#[derive(Clone)]
pub struct AppState {
    pub input_ctx: Arc<Mutex<Context>>,
}

#[derive(OpenApi)]
struct ApiDoc;

async fn api_error_middleware(req: axum::extract::Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().to_string();
    let res = next.run(req).await;
    let status = res.status();

    if !(status.is_client_error() || status.is_server_error()) {
        return res;
    }

    let (_, body) = res.into_parts();
    let mut stream = body.into_data_stream();
    let mut bytes = Vec::new();
    while let Some(chunk) = stream.next().await {
        if let Ok(chunk) = chunk {
            bytes.extend_from_slice(&chunk);
        }
    }
    let detail = String::from_utf8_lossy(&bytes).to_string();

    if status == StatusCode::NOT_FOUND {
        tracing::debug!(
            "HTTP {} {} -> {}: {}",
            method,
            uri,
            status.as_u16(),
            detail
        );
    } else if status.is_client_error() {
        tracing::warn!(
            "HTTP {} {} -> {}: {}",
            method,
            uri,
            status.as_u16(),
            detail
        );
    } else {
        tracing::error!(
            "HTTP {} {} -> {}: {}",
            method,
            uri,
            status.as_u16(),
            detail
        );
    }

    problem_details::ProblemDetails::from_status_code(status)
        .with_detail(detail)
        .into_response()
}

async fn cors_middleware(req: axum::extract::Request, next: Next) -> Response {
    let method = req.method().clone();
    let origin = req
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let requested_headers = req
        .headers()
        .get(header::ACCESS_CONTROL_REQUEST_HEADERS)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    let allow_origin = origin.as_deref().filter(|origin| {
        matches!(
            *origin,
            "https://steamloopback.host"
                | "http://steamloopback.host"
                | "http://localhost:5173"
        )
    });

    let is_preflight = method == Method::OPTIONS
        && req
            .headers()
            .contains_key(header::ACCESS_CONTROL_REQUEST_METHOD);

    if is_preflight {
        if let Some(allow_origin) = allow_origin {
            let mut res = Response::new(Body::empty());
            *res.status_mut() = StatusCode::NO_CONTENT;
            apply_cors_headers(&mut res, allow_origin, requested_headers.as_deref());
            return res;
        }

        if env::var("DEV") == Ok("1".to_string()) {
            let mut res = Response::new(Body::empty());
            *res.status_mut() = StatusCode::NO_CONTENT;
            apply_cors_headers(
                &mut res,
                "http://localhost:5173",
                requested_headers.as_deref(),
            );
            return res;
        }
    }

    let mut res = next.run(req).await;

    if let Some(allow_origin) = allow_origin {
        apply_cors_headers(&mut res, allow_origin, requested_headers.as_deref());
    } else if env::var("DEV") == Ok("1".to_string()) {
        apply_cors_headers(
            &mut res,
            "http://localhost:5173",
            requested_headers.as_deref(),
        );
    }

    res
}

fn apply_cors_headers(res: &mut Response, origin: &str, request_headers: Option<&str>) {
    if let Ok(header_value) = origin.parse() {
        res.headers_mut().insert(
            "Access-Control-Allow-Origin",
            header_value,
        );
    }

    res.headers_mut().insert("Vary", "Origin".parse().unwrap());
    res.headers_mut().insert(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
    );

    if let Some(request_headers) = request_headers {
        // CLIPPY!!!!
        if let Ok(header_value) = request_headers.parse() {
            res.headers_mut()
                .insert("Access-Control-Allow-Headers", header_value);
            return;
        }
    }

    res.headers_mut().insert(
        "Access-Control-Allow-Headers",
        "Content-Type".parse().unwrap(),
    );
}
