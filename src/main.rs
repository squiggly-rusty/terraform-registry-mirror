use std::{net::SocketAddr, path::PathBuf};

use axum::{
    extract::Path,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use terraform_registry_mirror::{
    redirect_to_real_download, MirrorVersionsList, ProviderMirror, ProviderPackage,
    ProviderPackageVersion, RealProviderRegistry,
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
    redirect_to_real_download(&package)
        .await
        .unwrap()
        .into_response()
}
