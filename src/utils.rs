use hmac::{Hmac, Mac};
use httpdate::fmt_http_date;
use log::{debug};
use reqwest::header::HeaderMap;
use sha2::{Digest, Sha256};

use std::str::{Split};
use std::time::SystemTime;
use base64::{Engine as _, engine::{general_purpose}};

use substring::Substring;
use url::Url;

use crate::models::EndPointParams;

type HmacSha256 = Hmac<Sha256>;

pub fn compute_content_sha256(content: &String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    return  general_purpose::STANDARD.encode(&result);
}

pub fn compute_signature(string_to_signed: &String, secret: &String) -> String {
    let mut mac = HmacSha256::new_from_slice(
        /*&base64::decode(secret).expect("HMAC compute decode secret failed")*/
        &general_purpose::STANDARD.decode(secret).expect("HMAC compute decode secret failed")
        ,
    )
    .expect("HMAC compuate_signature can take key of any size");

    mac.update(string_to_signed.as_bytes());

    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    return general_purpose::STANDARD.encode(code_bytes);
}

pub fn parse_endpoint(endpoint: &String) -> Result<EndPointParams, String> {
    debug!("{}", "parse_endpoint");
    let parameters: Split<&str> = endpoint.split(";");
    if parameters.clone().count() != 2 {
        return Err("Connection String Invalid".to_string());
    }
    let mut idx = 0_u8;
    let mut endpoint = EndPointParams {
        host_name: "".to_string(),
        access_key: "".to_string(),
    };

    for parameter in parameters.clone() {
        let param = parameter.clone();
        match idx {
            0 => {
                //get host name
                let endpoint_str = "endpoint=";
                if !param.starts_with(endpoint_str) {
                    return Err("Endpoint invalid".to_string());
                }
                let host = param.substring(endpoint_str.len(), param.len());
                let host = Url::parse(host).unwrap().host().unwrap().to_string();
                endpoint.host_name = host;
                debug!("end point > {}", endpoint.host_name);
            }
            1 => {
                //get access key
                let access_key_str = "accesskey=";
                if !param.starts_with(access_key_str) {
                    return Err("Access Key invalid".to_string());
                }
                let access_key = param.substring(access_key_str.len(), param.len());
                endpoint.access_key = access_key.to_string();
                debug!("access key > {}", endpoint.access_key);
            }
            _ => {}
        }
        idx = idx + 1;
    }
    Ok(endpoint)
}

pub fn get_request_header(
    url_endpoint: &Url,
    http_method: &str,
    request_id: &String,
    json_payload: &String,
    access_key: &String,
) -> Result<HeaderMap, String> {
    let mut header = HeaderMap::new();
    let compute_hash = compute_content_sha256(json_payload);
    //debug!("{:#?}",json_email_request);
    let now = SystemTime::now();
    let http_date = fmt_http_date(now);

    header.insert("Content-Type", "application/json".parse().unwrap());
    header.insert("repeatability-request-id", request_id.parse().unwrap());
    header.insert("repeatability-first-sent", http_date.parse().unwrap());
    header.insert("x-ms-date", http_date.clone().parse().unwrap());
    header.insert("x-ms-content-sha256", compute_hash.parse().unwrap());

    let host_authority = format!("{}", url_endpoint.host().unwrap(),);
    let path_and_query = format!("{}?{}", url_endpoint.path(), url_endpoint.query().unwrap());
    let string_to_sign = format!(
        "{}\n{}\n{};{};{}",
        http_method,
        path_and_query,
        http_date.clone(),
        host_authority,
        compute_hash.clone(),
    );
    debug!("{}\r\n", string_to_sign);

    let authorization = format!(
        "HMAC-SHA256 SignedHeaders=x-ms-date;host;x-ms-content-sha256&Signature={}",
        compute_signature(&string_to_sign.to_string(), &access_key.to_string())
    );
    header.insert("Authorization", authorization.parse().unwrap());
    debug!("{:#?}", header);

    Ok(header)
}
