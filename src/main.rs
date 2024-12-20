use axum::{
    extract::Path,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use terraform_registry_mirror::{
    MirrorVersionsList, PackageKind, ProviderMirror, RealProviderRegistry,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route(
            "/:hostname/:namespace/:package_name/index.json",
            get(list_available_versions),
        )
        .route(
            // NOTE: i couldn't find a way to match on :version.json, as it still grabbed everything. so if it's possible to split into two separate handlers it would be the best.
            "/:hostname/:namespace/:package_name/:version_or_path_part",
            get(handle_list_or_download),
        )
        .layer(TraceLayer::new_for_http());

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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

async fn handle_list_or_download(
    Path((hostname, namespace, package_name, version_or_path_part)): Path<(
        String,
        String,
        String,
        String,
    )>,
) -> Response {
    let mut registry = RealProviderRegistry {};

    if let Some(version) = version_or_path_part.strip_suffix(".json") {
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
    } else if let Some(download_url) = version_or_path_part.strip_suffix(".zip") {
        // TODO: for the inital case, this should be a temporary redirect to original location, see: https://docs.rs/axum/latest/axum/response/struct.Redirect.html
        todo!()
    } else {
        // TODO: this should be a proper error returned
        panic!("unsupported extension!")
    }
}
