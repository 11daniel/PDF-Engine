use axum::{Router, routing::post};

use crate::{
    controllers::pdf::{self},
    state::AppState,
};

pub fn get_router() -> Router<AppState> {
    Router::new().route("/generate-pdf", post(pdf::generate_pdf))
    //        .route("/mass-generate-pdf", post(mass_generate_pdf))
}
