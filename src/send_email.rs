use std::time::SystemTime;
use httpdate::fmt_http_date;
use log::{debug, error};
use reqwest::header::HeaderMap;
use url::Url;
use crate::models::{CommunicationErrorResponse, SentEmail};
use crate::utils::{compute_content_sha256, compute_signature};

pub async fn send_email(
    host_name: &String,
    access_key: &String,
    request_id: &String,
    request_email: &SentEmail,
) -> Result<String, String> {
    let url = format!(
        "https://{}/emails:send?api-version=2021-10-01-preview",
        host_name
    );

    //debug!("{}", url);
    let url_endpoint = Url::parse(url.as_str()).unwrap();
    debug!("{:#?}", url_endpoint);
    debug!("{:#?}", request_email);
    /*
     cal HMAC-SHA256
    */
    let json_email_request = serde_json::to_string(request_email).unwrap();

    let compute_hash = compute_content_sha256(json_email_request);
    //debug!("{:#?}",json_email_request);
    let client = reqwest::Client::new();

    let now = SystemTime::now();
    let http_date = fmt_http_date(now);

    let mut header = HeaderMap::new();
    header.insert("Content-Type", "application/json".parse().unwrap());
    header.insert("repeatability-request-id", request_id.parse().unwrap());
    header.insert("repeatability-first-sent", http_date.parse().unwrap());
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
        "POST\n{}\n{};{};{}",
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

    let resp = client
        .post(url)
        .headers(header)
        .json(request_email)
        .send()
        .await;

    if let Ok(resp) = resp {
        if resp.status().is_success() {
            debug!("{:#?}", resp);
            let message_header = resp.headers().get("x-ms-request-id");
            let mut message_id = "";
            if let Some(hv) = message_header {
                message_id = hv.to_str().unwrap();
            }
            Ok(message_id.to_string())
        }else{
            let error_reponse = resp.json::<CommunicationErrorResponse>().await;
            if let Ok(body) = error_reponse {
                error!("{:#?}", body);
                return Err(body.error.message);
            }else{
                return Err(error_reponse.err().unwrap().to_string())
            }
        }
    }else{
        return Err(resp.err().unwrap().to_string());
    }
}