use std::sync::Arc;

use crate::{env::Env, pdf::pool::PdfPool};

#[derive(Debug, Clone)]
pub struct AppState {
    pub env: Env,
    pub pdf_pool: Arc<PdfPool>,
    pub s3_client: aws_sdk_s3::Client,
}
