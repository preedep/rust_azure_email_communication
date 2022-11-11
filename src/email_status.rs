use std::time::SystemTime;
use httpdate::fmt_http_date;
use log::debug;
use reqwest::header::HeaderMap;
use url::Url;
use crate::models::EmailStatus;
use crate::utils::{compute_content_sha256, compute_signature};

pub async fn get_email_status(
    host_name: &String,
    access_key: &String,
    request_id: &String,
) -> Result<EmailStatus, Box<dyn std::error::Error>> {
    let url = format!(
        "https://{}/emails/{}/status?api-version=2021-10-01-preview",
        host_name, request_id
    );

    debug!("{}", url);

    let url_endpoint = Url::parse(url.as_str()).unwrap();
    debug!("{:#?}", url_endpoint);
    /*
     cal HMAC-SHA256
    */
    let json_email_request = String::new(); //serde_json::to_string(&email_request).unwrap();
    let compute_hash = compute_content_sha256(json_email_request);
    let client = reqwest::Client::new();

    let now = SystemTime::now();
    let http_date = fmt_http_date(now);

    let mut header = HeaderMap::new();
    header.insert("Content-Type", "application/json".parse().unwrap());
    header.insert("x-ms-date", http_date.clone().parse().unwrap());
    header.insert("x-ms-content-sha256", compute_hash.parse().unwrap());
    /*
    example :
    http://255.255.255.255:8080/
        Authority = 255.255.255.255:8080
        Host Name = 255.255.255.255
    */
    let host_authority = format!("{}", url_endpoint.host().unwrap(),);
    let path_and_query = format!("{}?{}", url_endpoint.path(), url_endpoint.query().unwrap());
    let string_to_sign = format!(
        "GET\n{}\n{};{};{}",
        path_and_query,
        http_date.clone(),
        host_authority,
        compute_hash.clone(),
    );
    debug!("{}\r\n", string_to_sign);

    let authorization = format!(
        "HMAC-SHA256 SignedHeaders=x-ms-date;host;x-ms-content-sha256&Signature={}",
        compute_signature(string_to_sign.to_string(), access_key.to_string())
    );
    //header.insert("host", host_authority.parse().unwrap());
    header.insert("Authorization", authorization.parse().unwrap());
    debug!("{:#?}", header);

    let resp = client.get(url).headers(header).send().await?;

    debug!("{:#?}", resp);
    if resp.status().is_success() {
        return Ok(resp.json::<EmailStatus>().await.unwrap());
    } else {
        if let Ok(body) = resp.text().await {
            debug!("{}", body);
        }
    }
    Ok(EmailStatus {
        message_id: "".to_string(),
        status: "".to_string(),
    })
}