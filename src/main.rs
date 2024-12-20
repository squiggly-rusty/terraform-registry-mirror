use std::{net::SocketAddr, path::PathBuf};

use axum::{
    extract::Path,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use terraform_registry_mirror::{
    redirect_to_real_download, MirrorVersionsList, PackageKind, ProviderMirror,
    RealProviderRegistry,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // configure certificate and private key used by https
    let from_pem_file = RustlsConfig::from_pem_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("certs")
            .join("localhost")
            .join("cert.pem"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("certs")
            .join("localhost")
            .join("key.pem"),
    );
    let config = from_pem_file.await.unwrap();

    let app = Router::new()
        .route(
            "/:hostname/:namespace/:package_name/index.json",
            get(list_available_versions),
        )
        .route(
            "/:hostname/:namespace/:package_name/:version.json",
            get(list_available_installation_packages),
        )
        .route(
            "/:hostname/:namespace/:package_name/:version/download/:os/:arch",
            get(download_package),
        )
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn list_available_versions(
    Path((hostname, namespace, package_name)): Path<(String, String, String)>,
) -> Json<MirrorVersionsList> {
    let mut registry = RealProviderRegistry {};
    return registry
        .list_versions(&hostname, &namespace, PackageKind::Provider, &package_name)
        .await
        .unwrap()
        .into();
}

async fn list_available_installation_packages(
    Path((hostname, namespace, package_name, version_part)): Path<(
        String,
        String,
        String,
        String,
    )>,
) -> Response {
    let mut registry = RealProviderRegistry {};

    if let Some(version) = version_part.strip_suffix(".json") {
        return Json(
            registry
                .list_installation_packages(
                    &hostname,
                    &namespace,
                    PackageKind::Provider,
                    &package_name,
                    version,
                )
                .await
                .unwrap(),
        )
        .into_response();
    } else {
        // TODO: this should be a proper error returned
        panic!("unsupported extension!")
    }
}

async fn download_package(
    Path((hostname, namespace, package_name, version, os, arch)): Path<(
        String,
        String,
        String,
        String,
        String,
        String,
    )>,
) -> Response {
    // TODO: this can be the place to fire off the download and then check on the next received request if we already have the file
    redirect_to_real_download(&hostname, &namespace, &package_name, &version, &os, &arch)
        .await
        .unwrap()
        .into_response()
}
