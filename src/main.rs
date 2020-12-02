mod error;
mod filter;

use crate::error::ServiceError;
use crate::filter::{Filter, FilterConfig};

use actix_web::{post, web, App, HttpRequest, HttpResponse, HttpServer};
use cloudevents::{AttributesReader, Data, Event, EventBuilder, EventBuilderV10};
use cloudevents_sdk_actix_web::{HttpRequestExt, HttpResponseBuilderExt};
use envconfig::Envconfig;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use url::Url;

#[derive(Clone, Debug, Envconfig)]
struct Config {
    #[envconfig(from = "BIND_ADDR", default = "127.0.0.1:8080")]
    pub bind_addr: String,
    #[envconfig(from = "CREDENTIALS_PATH", default = "/etc/config/credentials.json")]
    pub credentials_path: String,
    /// Allow to replace the data content type with `audio/wav` if it is a base type of `audio/vnd.wave`.
    #[envconfig(from = "FIX_WAV_TYPE", default = "true")]
    pub fix_wav_type: bool,
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
    url: Url,
    fix_wav_type: bool,
}

impl TryFrom<(reqwest::Client, Credentials, &Config)> for ApiClient {
    type Error = url::ParseError;

    fn try_from(from: (Client, Credentials, &Config)) -> Result<Self, Self::Error> {
        let url = Url::parse(&format!("{}/v1/recognize", from.1.url))?;
        Ok(ApiClient {
            client: from.0,
            credentials: from.1,
            url,
            fix_wav_type: from.2.fix_wav_type,
        })
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
        let content_type = if self.fix_wav_type && Self::needs_wav_fix(content_type.as_ref()) {
            "audio/wav"
        } else {
            content_type.as_ref()
        };

        Ok(self
            .client
            .post(self.url.clone())
            .header("Content-Type", content_type)
            .basic_auth("apikey", Some(self.credentials.apikey.clone()))
            .body(data)
            .send()
            .await?
            .json()
            .await?)
    }

    fn needs_wav_fix(content_type: &str) -> bool {
        match content_type.parse::<mime::Mime>() {
            Ok(mime) => match (mime.type_(), mime.subtype()) {
                (mime::AUDIO, sub) if sub == "vnd.wave" => true,
                _ => false,
            },
            Err(_) => false, // we don't know what it is, pass it on
        }
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
    filter: web::Data<Filter>,
) -> Result<HttpResponse, actix_web::Error> {
    let from = req.to_event(payload).await?;

    if !filter.test(&from) {
        log::debug!("Filter did not match, skip...");
        return Ok(HttpResponse::NoContent().finish());
    }

    let to = convert(from, &client).await?;

    Ok(HttpResponse::Ok().event(to).await?)
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = Config::init_from_env()?;

    let client = reqwest::ClientBuilder::new().build()?;
    let credentials: Credentials =
        serde_json::from_reader(std::fs::File::open(&config.credentials_path)?)?;

    let client: ApiClient = (client, credentials, &config).try_into()?;
    let filter: Filter = FilterConfig::init_from_env()?.into();

    HttpServer::new(move || {
        App::new()
            .wrap(actix_cors::Cors::permissive())
            .data(client.clone())
            .data(filter.clone())
            .service(post_event)
    })
    .bind(config.bind_addr)?
    .run()
    .await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_needs_wav_fix() {
        assert_eq!(ApiClient::needs_wav_fix(""), false);
        assert_eq!(ApiClient::needs_wav_fix("//////"), false);
        assert_eq!(ApiClient::needs_wav_fix("audio/wav"), false);
        assert_eq!(ApiClient::needs_wav_fix("audio/vnd.wave"), true);
        assert_eq!(ApiClient::needs_wav_fix("audio/vnd.wave; codec=1"), true);
    }
}
