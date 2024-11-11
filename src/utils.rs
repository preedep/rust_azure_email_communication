use crate::models::EndPointParams;
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use httpdate::fmt_http_date;
use log::debug;
use reqwest::header::HeaderMap;
use sha2::{Digest, Sha256};
use std::time::SystemTime;
use url::Url;

type HmacSha256 = Hmac<Sha256>;

/// Computes the SHA-256 hash of the given content and encodes it in base64.
///
/// # Arguments
///
/// * `content` - A string slice that holds the content to be hashed.
///
/// # Returns
///
/// * `String` - The base64 encoded SHA-256 hash of the content.
pub fn compute_content_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    general_purpose::STANDARD.encode(&result)
}

/// Computes the HMAC-SHA256 signature for the given string using the provided secret.
///
/// # Arguments
///
/// * `string_to_sign` - A string slice that holds the string to be signed.
/// * `secret` - A string slice that holds the secret key.
///
/// # Returns
///
/// * `Result<String, String>` - The base64 encoded HMAC-SHA256 signature or an error message.
pub fn compute_signature(string_to_sign: &str, secret: &str) -> Result<String, String> {
    let decoded_secret = general_purpose::STANDARD
        .decode(secret)
        .map_err(|e| format!("Failed to decode secret: {}", e))?;
    let mut mac = HmacSha256::new_from_slice(&decoded_secret)
        .map_err(|e| format!("Failed to create HMAC instance: {}", e))?;
    mac.update(string_to_sign.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    Ok(general_purpose::STANDARD.encode(code_bytes))
}

/// Parses the endpoint string and extracts the host name and access key.
///
/// # Arguments
///
/// * `endpoint` - A string slice that holds the endpoint string.
///
/// # Returns
///
/// * `Result<EndPointParams, String>` - The parsed endpoint parameters or an error message.
pub fn parse_endpoint(endpoint: &str) -> Result<EndPointParams, String> {
    debug!("Parsing endpoint");
    let parameters: Vec<&str> = endpoint.split(';').collect();
    if parameters.len() != 2 {
        return Err("Connection string must contain exactly two parameters".to_string());
    }

    let mut end_point_params = EndPointParams {
        host_name: String::new(),
        access_key: String::new(),
    };

    for param in parameters {
        if let Some(host) = param.strip_prefix("endpoint=") {
            let parsed_url =
                Url::parse(host).map_err(|e| format!("Invalid endpoint URL: {}", e))?;
            end_point_params.host_name = parsed_url
                .host_str()
                .ok_or_else(|| "Missing host in endpoint URL".to_string())?
                .to_string();
            debug!("Host name: {}", end_point_params.host_name);
        } else if let Some(key) = param.strip_prefix("accesskey=") {
            end_point_params.access_key = key.to_string();
            debug!("Access key: {}", end_point_params.access_key);
        } else {
            return Err("Invalid parameter in connection string".to_string());
        }
    }

    Ok(end_point_params)
}

/// Creates the request headers for the given parameters.
///
/// # Arguments
///
/// * `url_endpoint` - A reference to the `Url` struct representing the endpoint URL.
/// * `http_method` - A string slice that holds the HTTP method.
/// * `request_id` - A string slice that holds the request ID.
/// * `json_payload` - A string slice that holds the JSON payload.
/// * `access_key` - A string slice that holds the access key.
///
/// # Returns
///
/// * `Result<HeaderMap, String>` - The created request headers or an error message.
pub fn get_request_header(
    url_endpoint: &Url,
    http_method: &str,
    request_id: &str,
    json_payload: &str,
    access_key: &str,
) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    let content_hash = compute_content_sha256(json_payload);
    let now = SystemTime::now();
    let http_date = fmt_http_date(now);

    headers.insert("Content-Type", "application/json".parse().unwrap());
    headers.insert("repeatability-request-id", request_id.parse().unwrap());
    headers.insert("repeatability-first-sent", http_date.parse().unwrap());
    headers.insert("x-ms-date", http_date.parse().unwrap());
    headers.insert("x-ms-content-sha256", content_hash.parse().unwrap());

    let host_authority = url_endpoint
        .host_str()
        .ok_or_else(|| "Missing host in URL".to_string())?;
    let path_and_query = match url_endpoint.query() {
        Some(query) => format!("{}?{}", url_endpoint.path(), query),
        None => url_endpoint.path().to_string(),
    };
    let string_to_sign = format!(
        "{}\n{}\n{};{};{}",
        http_method, path_and_query, http_date, host_authority, content_hash
    );
    debug!("String to sign:\n{}", string_to_sign);

    let signature = compute_signature(&string_to_sign, access_key)?;
    let authorization = format!(
        "HMAC-SHA256 SignedHeaders=x-ms-date;host;x-ms-content-sha256&Signature={}",
        signature
    );
    headers.insert("Authorization", authorization.parse().unwrap());

    Ok(headers)
}