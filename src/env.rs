use std::env::{set_var, var};

use aws_config::Region;
use aws_sdk_s3::config::Credentials;

#[derive(Debug, Clone)]
pub struct Env {
    pub host: String,
    pub port: u16,
    pub log_level: String,
    pub s3_config: aws_sdk_s3::Config,
    pub s3_bucket: String,
    pub s3_public_url_format: String,
}

pub fn get_env() -> Env {
    let _ = dotenvy::dotenv();
    if var("RUST_LOG").is_err() {
        unsafe {
            set_var("RUST_LOG", "info");
        }
    }

    let s3_credentials = Credentials::new(
        var("S3_ACCESS_KEY").expect("Expected S3_ACCESS_KEY"),
        var("S3_SECRET_KEY").expect("Expected S3_SECRET_KEY"),
        None,
        None,
        "",
    );

    let s3_config = aws_sdk_s3::config::Builder::new()
        .region(Region::new("us-east-1"))
        .endpoint_url(var("S3_ENDPOINT").expect("Expected S3_ENDPOINT"))
        .credentials_provider(s3_credentials)
        .force_path_style(true)
        .build();

    Env {
        host: var("HOST").unwrap_or("0.0.0.0".into()),
        port: var("PORT")
            .unwrap_or("6970".into())
            .parse::<u16>()
            .expect("Invalid port specified!"),
        log_level: var("RUST_LOG").unwrap(), // we can unwrap here since RUST_LOG is guaranteed to
        // be set
        s3_config,
        s3_bucket: var("S3_BUCKET").expect("Expected S3_BUCKET!"),
        s3_public_url_format: var("S3_PUBLIC_URL_FORMAT").expect("Expected S3_PUBLIC_URL_FORMAT!"),
    }
}
