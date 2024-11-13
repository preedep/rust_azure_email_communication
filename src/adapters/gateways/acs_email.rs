// License: MIT
// This file is part of the Azure Communication Services Email Client Library, an open-source project.
// This source code is licensed under the MIT license found in the LICENSE file in the root directory of this source tree.

use crate::adapters::gateways::acs_shared_key::{get_request_header, parse_endpoint};
use crate::domain::entities::models::{
    EmailSendStatusType, ErrorDetail, ErrorResponse, SentEmail, SentEmailResponse,
};
use azure_core::auth::TokenCredential;
use azure_core::HttpClient;
use azure_identity::{create_credential, ClientSecretCredential};
use log::{debug, error};
use reqwest::header::RETRY_AFTER;
use reqwest::{Client, StatusCode};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::sleep;
use url::Url;
use uuid::Uuid;

type EmailResult<T> = Result<T, ErrorResponse>;
const API_VERSION: &str = "2023-01-15-preview";

// Azure Communication Services (ACS) authentication method
#[derive(Clone)]
enum ACSAuthMethod {
    SharedKey(String),
    ServicePrincipal {
        tenant_id: String,
        client_id: String,
        client_secret: String,
    },
    ManagedIdentity,
}

#[derive(Clone)]
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
    /// Send an email using the ACS client.
    ///
    /// # Arguments
    ///
    /// * `email` - A reference to the `SentEmail` struct containing the email details.
    ///
    /// # Returns
    ///
    /// * `EmailResult<String>` - The result of the email send operation, containing the message ID if successful.
    pub async fn send_email(&self, email: &SentEmail) -> EmailResult<String> {
        let request_id = format!("{}", Uuid::new_v4());
        acs_send_email(&self.host, &self.auth_method, request_id.as_str(), email).await
    }
    /// Sends an email using the ACS client and periodically checks the status, invoking a callback function with the status.
    ///
    /// # Arguments
    ///
    /// * `email` - A reference to the `SentEmail` struct containing the email details.
    /// * `call_back` - A callback function that takes the message ID, email send status, and optional error details.
    ///
    /// # Returns
    ///
    /// * `EmailResult<String>` - The result of the email send operation, containing the message ID if successful.
    #[allow(dead_code)]
    pub async fn send_email_with_callback<F>(
        self,
        email: &SentEmail,
        call_back: F,
    ) -> EmailResult<(String, oneshot::Receiver<()>)>
    where
        F: Fn(String, &EmailSendStatusType, Option<ErrorDetail>) + Send + Sync + 'static,
    {
        let request_id = format!("{}", Uuid::new_v4());
        let result =
            acs_send_email(&self.host, &self.auth_method, request_id.as_str(), email).await?;

        let message_id = result.clone();
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(5)).await;
                let resp_status = self.get_email_status(&message_id).await;
                if let Ok(status) = resp_status {
                    call_back(message_id.clone(), &status, None);
                    if matches!(
                        status,
                        EmailSendStatusType::Unknown
                            | EmailSendStatusType::Canceled
                            | EmailSendStatusType::Failed
                            | EmailSendStatusType::Succeeded
                    ) {
                        let _ = tx.send(());
                        break;
                    }
                } else {
                    call_back(
                        message_id.clone(),
                        &EmailSendStatusType::Failed,
                        Some(ErrorDetail {
                            message: Some(format!("Error getting email status: {:?}", resp_status)),
                            ..Default::default()
                        }),
                    );
                    let _ = tx.send(());
                    break;
                }
            }
        });

        Ok((result, rx))
    }

    /// Get the status of a sent email using the ACS client.
    ///
    /// # Arguments
    ///
    /// * `message_id` - A reference to the message ID string.
    ///
    /// # Returns
    ///
    /// * `EmailResult<EmailSendStatusType>` - The result of the email status query, containing the status if successful.
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
        acs_auth_method,
    )
    .await?;
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

/// Get an access token based on the provided authentication method.
///
/// # Arguments
///
/// * `auth_method` - A reference to the `ACSAuthMethod` enum specifying the authentication method.
///
/// # Returns
///
/// * `Result<String, String>` - The result of the token acquisition, containing the token if successful.
async fn get_access_token(auth_method: &ACSAuthMethod) -> Result<String, String> {
    match auth_method {
        ACSAuthMethod::ServicePrincipal {
            tenant_id,
            client_id,
            client_secret,
        } => {
            // Use Azure AD client credential flow (requires async-http-client support)
            let http_client = create_http_client();
            /*
            let token_url = format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
                tenant_id
            );*/
            let token_url = "https://login.microsoftonline.com/";
            debug!("Token URL: {}", token_url);
            debug!("Creating client secret credential");
            debug!("Client ID: {}", client_id);
            debug!("Client Secret: {}", client_secret);
            debug!("Tenant ID: {}", tenant_id);
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

/// Create headers for the request based on the provided authentication method.
///
/// # Arguments
///
/// * `url_endpoint` - A reference to the `Url` struct representing the endpoint URL.
/// * `method` - A reference to the HTTP method string.
/// * `request_id` - A reference to the request ID string.
/// * `json_body` - A reference to the JSON body string.
/// * `auth_method` - A reference to the `ACSAuthMethod` enum specifying the authentication method.
///
/// # Returns
///
/// * `EmailResult<reqwest::header::HeaderMap>` - The result of the header creation, containing the headers if successful.
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
            headers = get_request_header(url_endpoint, method, request_id, json_body, share_key)
                .map_err(|e| to_error_response("Header creation failed", e))?
        }
        ACSAuthMethod::ServicePrincipal { .. } | ACSAuthMethod::ManagedIdentity => {
            let token = get_access_token(auth_method)
                .await
                .map_err(|e| to_error_response("Failed to acquire access token", e))?;
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", token).parse().unwrap(),
            );
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                "application/json".parse().unwrap(),
            );
            headers.insert(
                reqwest::header::HeaderName::from_static("x-ms-client-request-id"),
                request_id.parse().unwrap(),
            );
        }
    }

    debug!("Headers: {:#?}", headers);
    Ok(headers)
}

/// Convert an error into an `ErrorResponse`.
///
/// # Arguments
///
/// * `message` - A reference to the error message string.
/// * `error` - An object that implements the `ToString` trait.
///
/// # Returns
///
/// * `ErrorResponse` - The error response containing the error details.
fn to_error_response(message: &str, error: impl ToString) -> ErrorResponse {
    ErrorResponse {
        error: Some(ErrorDetail {
            message: Some(format!("{}: {}", message, error.to_string())),
            ..Default::default()
        }),
    }
}

/// Get the status of a sent email using the ACS client.
///
/// # Arguments
///
/// * `host_name` - A reference to the host name string.
/// * `acs_auth_method` - A reference to the `ACSAuthMethod` enum specifying the authentication method.
/// * `request_id` - A reference to the request ID string.
///
/// # Returns
///
/// * `EmailResult<EmailSendStatusType>` - The result of the email status query, containing the status if successful.
async fn acs_get_email_status(
    host_name: &str,
    acs_auth_method: &ACSAuthMethod,
    request_id: &str,
) -> EmailResult<EmailSendStatusType> {
    let url = format!(
        "https://{}/emails/operations/{}?api-version={}",
        host_name.replace("https://", ""),
        request_id,
        API_VERSION
    );
    debug!("end point URL: {}", url);

    let response = send_request::<()>(
        reqwest::Method::GET,
        &url,
        request_id,
        None,
        acs_auth_method,
    )
    .await?;
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

/// Send an email using the ACS client.
///
/// # Arguments
///
/// * `host` - A reference to the host string.
/// * `acs_auth_method` - A reference to the `ACSAuthMethod` enum specifying the authentication method.
/// * `request_id` - A reference to the request ID string.
/// * `email` - A reference to the `SentEmail` struct containing the email details.
///
/// # Returns
///
/// * `EmailResult<String>` - The result of the email send operation, containing the message ID if successful.
async fn acs_send_email(
    host: &str,
    acs_auth_method: &ACSAuthMethod,
    request_id: &str,
    email: &SentEmail,
) -> EmailResult<String> {
    let url = format!(
        "https://{}/emails:send?api-version={}",
        host.replace("https://", ""),
        API_VERSION
    );
    debug!("end point URL: {}", url);
    let response = send_request(
        reqwest::Method::POST,
        &url,
        request_id,
        Some(email),
        acs_auth_method,
    )
    .await?;
    debug!("{:#?}", response);
    // handle response and retry if needed
    handle_response_and_retry_if_needed(
        response,
        reqwest::Method::POST,
        &url,
        request_id,
        Some(email),
        acs_auth_method,
        3,
    )
    .await
}
/// Handle the response from the email send operation and retry if needed.
///
/// # Arguments
///
/// * `response` - The `reqwest::Response` object.
/// * `method` - The HTTP method used for the request.
/// * `url` - The URL to send the request to.
/// * `request_id` - The request ID string.
/// * `body` - An optional reference to the request body.
/// * `acs_auth_method` - A reference to the `ACSAuthMethod` enum specifying the authentication method.
/// * `max_retries` - The maximum number of retries.
///
/// # Returns
///
/// * `EmailResult<String>` - The result of the response handling, containing the message ID if successful.
async fn handle_response_and_retry_if_needed<T>(
    mut response: reqwest::Response,
    method: reqwest::Method,
    url: &str,
    request_id: &str,
    body: Option<&T>,
    acs_auth_method: &ACSAuthMethod,
    max_retries: u32,
) -> EmailResult<String>
where
    T: serde::Serialize,
{
    let mut retries = 0;

    loop {
        match response.status() {
            StatusCode::ACCEPTED => {
                return parse_response::<SentEmailResponse>(response)
                    .await?
                    .id
                    .ok_or_else(create_missing_id_error);
            }
            StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE => {
                if retries >= max_retries {
                    error!("Max retries exceeded");
                    return parse_error_response(response).await;
                }

                if let Some(retry_after) = response.headers().get(RETRY_AFTER) {
                    if let Ok(retry_after_value) = retry_after.to_str() {
                        if let Ok(retry_after_secs) = retry_after_value.parse::<u64>() {
                            debug!("Retrying after {} seconds", retry_after_secs);
                            sleep(Duration::from_secs(retry_after_secs)).await;
                        } else {
                            error!("Failed to parse Retry-After header value");
                            return parse_error_response(response).await;
                        }
                    } else {
                        error!("Failed to parse Retry-After header value");
                        return parse_error_response(response).await;
                    }
                } else {
                    // Implement exponential backoff
                    let backoff_secs = 2u64.pow(retries);
                    debug!(
                        "Retry-After header not found. Retrying after {} seconds",
                        backoff_secs
                    );
                    sleep(Duration::from_secs(backoff_secs)).await;
                }

                retries += 1;

                // Retry the request
                let new_response =
                    send_request(method.clone(), url, request_id, body, acs_auth_method).await?;
                response = new_response;
            }
            _ => {
                error!("Failed to send email: {:#?}", response);
                return parse_error_response(response).await;
            }
        }
    }
}

/// Parse the response from the email send operation.
///
/// # Arguments
///
/// * `response` - The `reqwest::Response` object.
///
/// # Returns
///
/// * `EmailResult<T>` - The result of the response parsing, containing the parsed response if successful.
async fn parse_response<T>(response: reqwest::Response) -> EmailResult<T>
where
    T: serde::de::DeserializeOwned,
{
    response
        .json::<T>()
        .await
        .map_err(|e| to_error_response("Failed to parse response", e))
}

/// Parse the error response from the email send operation.
///
/// # Arguments
///
/// * `response` - The `reqwest::Response` object.
///
/// # Returns
///
/// * `EmailResult<String>` - The result of the error response parsing, containing the error response if successful.
async fn parse_error_response(response: reqwest::Response) -> EmailResult<String> {
    let error_response = parse_response::<ErrorResponse>(response).await?;
    Err(error_response)
}

/// Create an error response for a missing status.
///
/// # Returns
///
/// * `ErrorResponse` - The error response indicating a missing status.
fn create_missing_status_error() -> ErrorResponse {
    to_error_response("Missing status in response", "")
}

/// Create an error response for a missing ID.
///
/// # Returns
///
/// * `ErrorResponse` - The error response indicating a missing ID.
fn create_missing_id_error() -> ErrorResponse {
    to_error_response("Missing ID in response", "")
}
