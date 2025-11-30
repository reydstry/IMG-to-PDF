use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;
use tracing::info;

mod handlers;
mod pdf;
mod image_utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting Image to PDF API Server...");

    // Membangun router
    let  app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/convert", post(handlers::convert_images_to_pdf))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
    .await
    .unwrap();

    info!("✅ Server running on http://localhost:3000");
    info!("📝 Endpoints:");
    info!("   GET  /health          - Health check");
    info!("   POST /api/convert     - Convert images to PDF");
    info!("");
    info!("🧪 Test with Postman:");
    info!("   Method: POST");
    info!("   URL: http://localhost:3000/api/convert");
    info!("   Body: form-data");
    info!("   Key: images (type: File) - pilih multiple files");

    // Start server
    axum::serve(listener, app).await.unwrap();
}