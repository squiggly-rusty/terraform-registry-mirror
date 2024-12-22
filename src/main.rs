use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use terraform_registry_mirror::{
    redirect_to_real_download, LocalStorageBackend, MirrorVersionsList, ProviderMirror,
    ProviderPackage, ProviderPackageVersion, RealProviderRegistry, StorageBackend,
};
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Clone)]
struct AppState {
    storage_backend: Arc<LocalStorageBackend>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
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

    let state = AppState {
        storage_backend: LocalStorageBackend::new().into(),
    };
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
        .with_state(state)
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
    let registry = RealProviderRegistry {};
    let package = ProviderPackage::new(&hostname, &namespace, &package_name);
    return registry.list_versions(&package).await.unwrap().into();
}

async fn list_available_installation_packages(
    Path((hostname, namespace, package_name, version_part)): Path<(String, String, String, String)>,
) -> Response {
    let registry = RealProviderRegistry {};

    if let Some(version) = version_part.strip_suffix(".json") {
        let package = ProviderPackage::new(&hostname, &namespace, &package_name);
        return Json(
            registry
                .list_installation_packages(&package, version)
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
    State(state): State<AppState>,
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
    let version = ProviderPackageVersion::new(version, os, arch);
    let package = ProviderPackage::with_version(&hostname, &namespace, &package_name, version);

    info!("in download_package!");
    if let Some(uri) = state.storage_backend.return_package_link(package.clone()) {
        info!("package available, returning link from storage!");
        // TODO: or we can return back a file or start streaming here
        Redirect::temporary(&uri).into_response()
    } else {
        info!("package not available, returning real download link!");
        redirect_to_real_download(&package)
            .await
            .unwrap()
            .into_response()
    }
}
