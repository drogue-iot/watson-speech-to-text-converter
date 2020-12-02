use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum ServiceError {
    #[snafu(display("Invalid request: {}", details))]
    InvalidRequest { details: String },
    #[snafu(display("Event builder error: {}", source))]
    EventBuilderError {
        source: cloudevents::event::EventBuilderError,
    },
    #[snafu(display("Client Error: {}", source))]
    ClientError { source: reqwest::Error },
}

impl ServiceError {
    pub fn name(&self) -> &str {
        match self {
            ServiceError::InvalidRequest { .. } => "InvalidRequest",
            ServiceError::EventBuilderError { .. } => "EventBuilderError",
            ServiceError::ClientError { .. } => "ClientError",
        }
    }

    pub fn invalid_request<S>(s: S) -> Self
    where
        S: Into<String>,
    {
        ServiceError::InvalidRequest { details: s.into() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl ResponseError for ServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::InvalidRequest { .. } => StatusCode::NOT_ACCEPTABLE,
            ServiceError::EventBuilderError { .. } => StatusCode::NOT_ACCEPTABLE,
            ServiceError::ClientError { source } => {
                source.status().unwrap_or(StatusCode::BAD_REQUEST)
            }
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            message: self.to_string(),
            error: self.name().into(),
        };
        HttpResponse::build(status_code).json(error_response)
    }
}

impl From<cloudevents::event::EventBuilderError> for ServiceError {
    fn from(err: cloudevents::event::EventBuilderError) -> Self {
        ServiceError::EventBuilderError { source: err }
    }
}

impl From<reqwest::Error> for ServiceError {
    fn from(err: reqwest::Error) -> Self {
        ServiceError::ClientError { source: err }
    }
}
