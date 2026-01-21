use std::{error::Error, fmt::Display};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub type BoxedError = anyhow::Error;

#[derive(Debug)]
pub struct GenericError(pub String);

impl Display for GenericError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)?;
        Ok(())
    }
}

impl Error for GenericError {}

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(
                "Something went wrong: {:?}\nTrace:\n{:?}",
                self.0,
                self.0.backtrace()
            ),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
