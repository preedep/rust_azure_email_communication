use crate::models::{
    EmailSendStatusType, ErrorDetail, ErrorResponse, SentEmail, SentEmailResponse,
};
use crate::utils::get_request_header;
use log::debug;
use reqwest::{Client, StatusCode};
use url::Url;

type EmailResult<T> = Result<T, ErrorResponse>;

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
    let url_endpoint = Url::parse(url).map_err(|e| ErrorResponse {
        error: Some(ErrorDetail {
            message: Some(format!("Invalid URL: {}", e)),
            ..Default::default()
        }),
    })?;

    let client = Client::new();
    let json_body = if let Some(body) = body {
        serde_json::to_string(body).map_err(|e| ErrorResponse {
            error: Some(ErrorDetail {
                message: Some(format!("Failed to serialize request body: {}", e)),
                ..Default::default()
            }),
        })?
    } else {
        String::new()
    };

    let headers = get_request_header(
        &url_endpoint,
        method.as_str(),
        &request_id.to_string(),
        &json_body,
        &access_key.to_string(),
    )
    .map_err(|e| ErrorResponse {
        error: Some(ErrorDetail {
            message: Some(format!("Header creation failed: {}", e)),
            ..Default::default()
        }),
    })?;

    let request_builder = client.request(method, url).headers(headers);
    let request_builder = if let Some(body) = body {
        request_builder.json(body)
    } else {
        request_builder
    };

    request_builder.send().await.map_err(|e| ErrorResponse {
        error: Some(ErrorDetail {
            message: Some(format!("Request failed: {}", e)),
            ..Default::default()
        }),
    })
}

pub async fn get_email_status(
    host_name: &str,
    access_key: &str,
    request_id: &str,
) -> EmailResult<EmailSendStatusType> {
    let url = format!(
        "https://{}/emails/operations/{}?api-version=2023-01-15-preview",
        host_name, request_id,
    );

    let response =
        send_request::<()>(reqwest::Method::GET, &url, access_key, request_id, None).await?;

    if response.status() == StatusCode::OK {
        let email_resp = response
            .json::<SentEmailResponse>()
            .await
            .map_err(|e| ErrorResponse {
                error: Some(ErrorDetail {
                    message: Some(format!("Failed to parse response: {}", e)),
                    ..Default::default()
                }),
            })?;
        email_resp
            .status
            .map(|status| Ok(status.to_type()))
            .unwrap_or_else(|| {
                Err(ErrorResponse {
                    error: Some(ErrorDetail {
                        message: Some("Missing status in response".to_string()),
                        ..Default::default()
                    }),
                })
            })
    } else {
        let error_resp = response
            .json::<ErrorResponse>()
            .await
            .map_err(|e| ErrorResponse {
                error: Some(ErrorDetail {
                    message: Some(format!("Failed to parse error response: {}", e)),
                    ..Default::default()
                }),
            })?;
        Err(error_resp)
    }
}

pub async fn send_email(
    host_name: &str,
    access_key: &str,
    request_id: &str,
    request_email: &SentEmail,
) -> EmailResult<String> {
    let url = format!(
        "https://{}/emails:send?api-version=2023-01-15-preview",
        host_name
    );

    let response = send_request(
        reqwest::Method::POST,
        &url,
        access_key,
        request_id,
        Some(request_email),
    )
    .await?;

    debug!("{:#?}", response);
    if response.status() == StatusCode::ACCEPTED {
        let email_resp = response
            .json::<SentEmailResponse>()
            .await
            .map_err(|e| ErrorResponse {
                error: Some(ErrorDetail {
                    message: Some(format!("Failed to parse response: {}", e)),
                    ..Default::default()
                }),
            })?;
        email_resp.id.map(Ok).unwrap_or_else(|| {
            Err(ErrorResponse {
                error: Some(ErrorDetail {
                    message: Some("Missing ID in response".to_string()),
                    ..Default::default()
                }),
            })
        })
    } else {
        let error_resp = response
            .json::<ErrorResponse>()
            .await
            .map_err(|e| ErrorResponse {
                error: Some(ErrorDetail {
                    message: Some(format!("Failed to parse error response: {}", e)),
                    ..Default::default()
                }),
            })?;
        Err(error_resp)
    }
}
