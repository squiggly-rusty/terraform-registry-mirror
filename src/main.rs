use axum::{routing::get, Router, extract::Path};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build our application with a single route
    let app = Router::new()
        .route(":hostname/:namespace/:package_name/index.json", get(mock_versions))
        .layer(
            TraceLayer::new_for_http(),
        );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn mock_versions(Path((hostname, namespace, package_name)): Path<(String, String, String)>) -> String {
    return check_package_versions(namespace, PackageKind.Provider, package_name);
}
