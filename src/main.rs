use std::sync::Arc;

use axum::{Router, routing::get};
use pdfsnap_server::env::get_env;
use pdfsnap_server::pdf::pool::PdfPool;
use tokio::net::TcpListener;

use pdfsnap_server::state::AppState;

#[tokio::main]
async fn main() {
    let env = get_env();

    env_logger::init();

    let bind_string = format!("{}:{}", &env.host, &env.port);

    let pdf_pool = Arc::new(PdfPool::new(4));

    let s3_client = aws_sdk_s3::Client::from_conf(env.s3_config.clone());
    
    log::debug!("Using environment: {:#?}", env);

    let state = AppState {
        env,
        pdf_pool,
        s3_client,
    };

    let app: Router = Router::new()
        .route("/healthcheck", get(pdfsnap_server::controllers::healthcheck))
        .nest("/api/v0/pdf", pdfsnap_server::routers::pdf::get_router())
        .with_state(state);

    let listener = TcpListener::bind(&bind_string).await.unwrap();
    log::info!("Listening on {}", &bind_string);

    axum::serve(listener, app).await.unwrap();
}
