mod error;

use crate::error::ServiceError;

use actix_web::{post, web, App, HttpRequest, HttpResponse, HttpServer};
use cloudevents::{AttributesReader, Data, Event, EventBuilder, EventBuilderV10};
use cloudevents_sdk_actix_web::{HttpRequestExt, HttpResponseBuilderExt};
use envconfig::Envconfig;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Envconfig)]
struct Config {
    #[envconfig(from = "BIND_ADDR", default = "127.0.0.1:8080")]
    pub bind_addr: String,
    #[envconfig(from = "CREDENTIALS_PATH", default = "/etc/config/credentials.json")]
    pub credentials_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Credentials {
    pub apikey: String,
    pub url: String,
}

#[derive(Clone, Debug)]
struct ApiClient {
    client: reqwest::Client,
    credentials: Credentials,
}

impl From<(reqwest::Client, Credentials)> for ApiClient {
    fn from(from: (Client, Credentials)) -> Self {
        ApiClient {
            client: from.0,
            credentials: from.1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RecognizeResult {}

#[derive(Debug, Serialize, Deserialize)]
struct RecognizeResults {}

impl ApiClient {
    pub async fn recognize<S: AsRef<str>>(
        &self,
        content_type: S,
        data: Vec<u8>,
    ) -> Result<serde_json::Value, ServiceError> {
        Ok(self
            .client
            .post(&self.credentials.url)
            .header("Content-Type", content_type.as_ref())
            .basic_auth("apikey", Some(self.credentials.apikey.clone()))
            .body(data)
            .send()
            .await?
            .json()
            .await?)
    }
}

/// The actual converter
async fn convert(event: Event, client: &ApiClient) -> Result<Event, ServiceError> {
    let content_type = event
        .datacontenttype()
        .ok_or_else(|| ServiceError::invalid_request("Missing data-content-type"))?;

    let result = match event.data() {
        Some(Data::Binary(blob)) => Ok(client.recognize(content_type, blob.clone()).await?),
        _ => Err(ServiceError::invalid_request(
            "Wrong data type, must be 'binary'",
        )),
    }?;

    log::debug!("Recognized: {:?}", result);

    Ok(EventBuilderV10::from(event)
        .data_with_schema(
            "application/json",
            "https://cloud.ibm.com/services/speech-to-text/v1/recognize",
            result,
        )
        .build()?)
}

#[post("/")]
async fn post_event(
    req: HttpRequest,
    payload: web::Payload,
    client: web::Data<ApiClient>,
) -> Result<HttpResponse, actix_web::Error> {
    let from = req.to_event(payload).await?;
    let to = convert(from, &client).await?;

    Ok(HttpResponse::Ok().event(to).await?)
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = Config::init_from_env()?;

    let client = reqwest::ClientBuilder::new().build()?;
    let credentials: Credentials =
        serde_json::from_reader(std::fs::File::open(config.credentials_path)?)?;

    let client: ApiClient = (client, credentials).into();

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_cors::Cors::permissive())
            .data(client.clone())
            .service(post_event)
    })
    .bind(config.bind_addr)?
    .run()
    .await?;

    Ok(())
}
