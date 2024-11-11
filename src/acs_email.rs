use crate::models::{
    EmailSendStatusType, ErrorDetail, ErrorResponse, SentEmail, SentEmailResponse,
};
use crate::utils::{get_request_header, parse_endpoint};
use azure_core::auth::TokenCredential;
use azure_core::HttpClient;
use azure_identity::{
    create_credential, create_default_credential, ClientSecretCredential, DefaultAzureCredential,
    DefaultAzureCredentialBuilder,
};
use std::sync::Arc;

use log::debug;
use openssl::ssl::ConnectConfiguration;
use reqwest::{Client, StatusCode};
use url::Url;
use uuid::Uuid;

type EmailResult<T> = Result<T, ErrorResponse>;
const API_VERSION: &str = "2023-01-15-preview";

// Azure Communication Services (ACS) authentication method
enum ACSAuthMethod {
    SharedKey(String),
    ServicePrincipal {
        tenant_id: String,
        client_id: String,
        client_secret: String,
    },
    ManagedIdentity,
}

pub struct ACSClient {
    host: String,
    auth_method: ACSAuthMethod,
}

pub struct ACSClientBuilder {
    host: Option<String>,
    connection_string: Option<String>,
    auth_method: Option<ACSAuthMethod>,
}

impl ACSClientBuilder {
    // Create a new builder instance
    pub fn new() -> Self {
        ACSClientBuilder {
            host: None,
            connection_string: None,
            auth_method: None,
        }
    }

    // Set the host for the client
    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }

    // Set the authentication method for the client using a shared key
    pub fn connection_string(mut self, connection_string: &str) -> Self {
        self.connection_string = Some(connection_string.to_string());
        self
    }

    // Set the authentication method for the client using a service principal
    pub fn service_principal(
        mut self,
        tenant_id: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Self {
        self.auth_method = Some(ACSAuthMethod::ServicePrincipal {
            tenant_id: tenant_id.to_string(),
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        });
        self
    }

    // Set the authentication method for the client using managed identity
    pub fn managed_identity(mut self) -> Self {
        self.auth_method = Some(ACSAuthMethod::ManagedIdentity);
        self
    }

    // Build and return the ACSClient
    pub fn build(self) -> Result<ACSClient, String> {
        if let Some(connection_string) = self.connection_string {
            let parsed_res = parse_endpoint(&connection_string)
                .map_err(|e| format!("Failed to parse connection string: {}", e))?;
            let host = parsed_res.host_name;
            let auth_method = ACSAuthMethod::SharedKey(parsed_res.access_key);
            return Ok(ACSClient { host, auth_method });
        }

        let host = self.host.ok_or_else(|| "Host is required".to_string())?;
        let auth_method = self
            .auth_method
            .ok_or_else(|| "Authentication method is required".to_string())?;
        Ok(ACSClient { host, auth_method })
    }
}

impl ACSClient {
    pub async fn send_email(&self, email: &SentEmail) -> EmailResult<String> {
        let request_id = format!("{}", Uuid::new_v4());
        acs_send_email(&self.host, &self.auth_method, request_id.as_str(), email).await
    }
    pub async fn get_email_status(&self, message_id: &str) -> EmailResult<EmailSendStatusType> {
        acs_get_email_status(&self.host, &self.auth_method, message_id).await
    }
}

async fn send_request<T>(
    method: reqwest::Method,
    url: &str,
    request_id: &str,
    body: Option<&T>,
    acs_auth_method: &ACSAuthMethod,
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
        acs_auth_method
    ).await?;
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
// Adding a function to create a `HttpClient`
fn create_http_client() -> Arc<dyn HttpClient> {
    // Assuming `request` is used as the HTTP client
    Arc::new(Client::new()) as Arc<dyn HttpClient>
}

async fn get_access_token(auth_method: &ACSAuthMethod) -> Result<String, String> {
    match auth_method {
        ACSAuthMethod::ServicePrincipal {
            tenant_id,
            client_id,
            client_secret,
        } => {
            // Use Azure AD client credential flow (requires async-http-client support)
            let http_client = create_http_client();
            let token_url = format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
                tenant_id
            );
            debug!("Token URL: {}", token_url);
            let credential = ClientSecretCredential::new(
                http_client,
                Url::parse(&token_url).unwrap(),
                tenant_id.to_string(),
                client_id.to_string(),
                client_secret.to_string(),
            );
            let token = credential
                .get_token(&["https://communication.azure.com/.default"])
                .await
                .map_err(|e| format!("Failed to get access token: {}", e))?;
            debug!("Access token: {:#?}", token);

            return Ok(token.token.secret().to_owned());
        }
        ACSAuthMethod::ManagedIdentity => {
            let credential =
                create_credential().map_err(|e| format!("Failed to create credential: {}", e))?;
            let token = credential
                .get_token(&["https://communication.azure.com/.default"])
                .await
                .map_err(|e| format!("Failed to get access token: {}", e))?;
            return Ok(token.token.secret().to_owned());
        }
        _ => {}
    }
    Ok("".to_string())
}

async fn create_headers(
    url_endpoint: &Url,
    method: &str,
    request_id: &str,
    json_body: &str,
    auth_method: &ACSAuthMethod,
) -> EmailResult<reqwest::header::HeaderMap> {
    let mut headers = reqwest::header::HeaderMap::new();

    match auth_method {
        ACSAuthMethod::SharedKey(share_key) => {
            headers = get_request_header(
                url_endpoint,
                method,
                request_id,
                json_body,
                share_key,
            )
                .map_err(|e| to_error_response("Header creation failed", e))?
        }
        ACSAuthMethod::ServicePrincipal { .. } | ACSAuthMethod::ManagedIdentity => {
            let token = get_access_token(auth_method).await
                .map_err(|e| to_error_response("Failed to acquire access token", e))?;
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", token).parse().unwrap(),
            );
        }
    }

    headers.insert(
        reqwest::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("x-ms-client-request-id"),
        request_id.parse().unwrap(),
    );

    Ok(headers)
}

fn to_error_response(message: &str, error: impl ToString) -> ErrorResponse {
    ErrorResponse {
        error: Some(ErrorDetail {
            message: Some(format!("{}: {}", message, error.to_string())),
            ..Default::default()
        }),
    }
}
async fn acs_get_email_status(
    host_name: &str,
    acs_auth_method: &ACSAuthMethod,
    request_id: &str,
) -> EmailResult<EmailSendStatusType> {
    let access_key = match acs_auth_method {
        ACSAuthMethod::SharedKey(key) => key,
        _ => panic!("Shared key is required for getting email status"),
    };
    let url = format!(
        "https://{}/emails/operations/{}?api-version={}",
        host_name, request_id, API_VERSION
    );
    let response =
        send_request::<()>(reqwest::Method::GET, &url, request_id, None, acs_auth_method).await?;
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

async fn acs_send_email(
    host: &str,
    acs_auth_method: &ACSAuthMethod,
    request_id: &str,
    email: &SentEmail,
) -> EmailResult<String> {

    let url = format!("https://{}/emails:send?api-version={}", host, API_VERSION);
    let response = send_request(
        reqwest::Method::POST,
        &url,
        request_id,
        Some(email),
        acs_auth_method,
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
