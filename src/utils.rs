use hmac::{Hmac, Mac};
use httpdate::fmt_http_date;
use log::{debug};
use reqwest::header::HeaderMap;
use sha2::{Digest, Sha256};
use std::{ fmt};
use std::str::{FromStr, Split};
use std::time::SystemTime;
use substring::Substring;
use url::Url;


use crate::models::EndPointParams;

type HmacSha256 = Hmac<Sha256>;

pub fn compute_content_sha256(content: String) -> String {

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    return base64::encode(&result);
}

pub fn compute_signature(string_to_signed: String, secret: String) -> String {
    let mut mac = HmacSha256::new_from_slice(
        &base64::decode(secret).expect("HMAC compute decode secret failed"),
    )
        .expect("HMAC compuate_signature can take key of any size");

    mac.update(string_to_signed.as_bytes());

    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    return base64::encode(code_bytes);
}

pub fn parse_endpoint(endpoint: String) -> Result<EndPointParams, String> {
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