use clap::ValueEnum;
use crate::models::{
    EmailSendStatusType, ErrorDetail, ErrorResponse, SentEmail, SentEmailResponse,
};
use crate::utils::get_request_header;
use log::debug;
use reqwest::{Client, StatusCode};
use url::Url;

type EmailResult<T> = Result<T, ErrorResponse>;
const API_VERSION: &str = "2023-01-15-preview";


// Define the AuthenticationMethod enum
#[derive(Debug, Clone, ValueEnum)]
pub enum AuthenticationMethod {
    ManagedIdentity,
    // Define the ServicePrincipal enum
    // This enum is used to specify the authentication method when using a service principal
    // ClientId: The client ID of the service principal
    // ClientSecret: The client secret of the service principal
    // TenantId: The tenant ID of the service principal
    ServicePrincipal,
    // Define the SharedKey enum
    // This enum is used to specify the authentication method when using a shared key
    // SharedKey: The shared key
    SharedKey,
}

// Define the ACSProtocol enum
#[derive(Debug, Clone, ValueEnum)]
pub enum ACSProtocol {
    REST,
    SMTP,
}

async fn send_request<T>(
    method: reqwest::Method,
    url: &str,
    access_key: &str,
    request_id: &str,
    body: Option<&T>,
) -> EmailResult<reqwest::Response>
where
    T: serde::Serialize,
{
    let url_endpoint = parse_url(url)?;
    let client = Client::new();
    let json_body = serialize_body(body)?;
    let headers = create_headers(
        &url_endpoint,
        method.as_str(),
        request_id,
        &json_body,
        access_key,
    )?;
    let request_builder = client.request(method, url).headers(headers);
    let request_builder = if let Some(body) = body {
        request_builder.json(body)
    } else {
        request_builder
    };
    request_builder
        .send()
        .await
        .map_err(|e| to_error_response("Request failed", e))
}

fn parse_url(url: &str) -> EmailResult<Url> {
    Url::parse(url).map_err(|e| to_error_response("Invalid URL", e))
}

fn serialize_body<T: serde::Serialize>(body: Option<&T>) -> EmailResult<String> {
    if let Some(body) = body {
        serde_json::to_string(body)
            .map_err(|e| to_error_response("Failed to serialize request body", e))
    } else {
        Ok(String::new())
    }
}

fn create_headers(
    url_endpoint: &Url,
    method: &str,
    request_id: &str,
    json_body: &str,
    access_key: &str,
) -> EmailResult<reqwest::header::HeaderMap> {
    get_request_header(
        url_endpoint,
        method,
        &request_id.to_string(),
        json_body,
        &access_key.to_string(),
    )
    .map_err(|e| to_error_response("Header creation failed", e))
}

fn to_error_response(message: &str, error: impl ToString) -> ErrorResponse {
    ErrorResponse {
        error: Some(ErrorDetail {
            message: Some(format!("{}: {}", message, error.to_string())),
            ..Default::default()
        }),
    }
}

pub async fn get_email_status(
    host_name: &str,
    access_key: &str,
    request_id: &str,
) -> EmailResult<EmailSendStatusType> {
    let url = format!(
        "https://{}/emails/operations/{}?api-version={}",
        host_name, request_id, API_VERSION
    );
    let response =
        send_request::<()>(reqwest::Method::GET, &url, access_key, request_id, None).await?;
    if response.status() == StatusCode::OK {
        let email_response = parse_response::<SentEmailResponse>(response).await?;
        email_response
            .status
            .map(|status| Ok(status.to_type()))
            .unwrap_or_else(|| Err(create_missing_status_error()))
    } else {
        let error_response = parse_response::<ErrorResponse>(response).await?;
        Err(error_response)
    }
}

pub async fn send_email(
    host: &str,
    access_key: &str,
    request_id: &str,
    email: &SentEmail,
) -> EmailResult<String> {
    let url = format!("https://{}/emails:send?api-version={}", host, API_VERSION);
    let response = send_request(
        reqwest::Method::POST,
        &url,
        access_key,
        request_id,
        Some(email),
    )
    .await?;
    debug!("{:#?}", response);
    handle_response(response).await
}

async fn handle_response(response: reqwest::Response) -> EmailResult<String> {
    if response.status() == StatusCode::ACCEPTED {
        parse_response::<SentEmailResponse>(response)
            .await?
            .id
            .ok_or_else(create_missing_id_error)
    } else {
        parse_error_response(response).await
    }
}

async fn parse_response<T>(response: reqwest::Response) -> EmailResult<T>
where
    T: serde::de::DeserializeOwned,
{
    response
        .json::<T>()
        .await
        .map_err(|e| to_error_response("Failed to parse response", e))
}

async fn parse_error_response(response: reqwest::Response) -> EmailResult<String> {
    let error_response = parse_response::<ErrorResponse>(response).await?;
    Err(error_response)
}

fn create_missing_status_error() -> ErrorResponse {
    to_error_response("Missing status in response", "")
}

fn create_missing_id_error() -> ErrorResponse {
    to_error_response("Missing ID in response", "")
}
