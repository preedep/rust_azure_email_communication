use hmac::{Hmac, Mac};
use httpdate::fmt_http_date;
use log::{debug, error, info};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{env, fmt};
use std::str::{FromStr, Split};
use std::time::SystemTime;
use substring::Substring;
use url::Url;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct SentEmail {
    #[serde(rename = "headers")]
    headers: Option<Vec<Header>>,

    #[serde(rename = "sender")]
    sender: Option<String>,

    #[serde(rename = "content")]
    content: Option<Content>,

    #[serde(rename = "importance")]
    importance: Option<String>,

    #[serde(rename = "recipients")]
    recipients: Option<Recipients>,

    #[serde(rename = "attachments")]
    attachments: Option<Vec<Attachment>>,

    #[serde(rename = "replyTo")]
    reply_to: Option<Vec<ReplyTo>>,

    #[serde(rename = "disableUserEngagementTracking")]
    disable_user_engagement_tracking: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attachment {
    #[serde(rename = "name")]
    name: Option<String>,

    #[serde(rename = "attachmentType")]
    attachment_type: Option<String>,

    #[serde(rename = "contentBytesBase64")]
    content_bytes_base64: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    #[serde(rename = "subject")]
    subject: Option<String>,

    #[serde(rename = "plainText")]
    plain_text: Option<String>,

    #[serde(rename = "html")]
    html: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Header {
    #[serde(rename = "name")]
    name: Option<String>,

    #[serde(rename = "value")]
    value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Recipients {
    #[serde(rename = "to")]
    to: Option<Vec<ReplyTo>>,

    #[serde(rename = "CC")]
    cc: Option<Vec<ReplyTo>>,

    #[serde(rename = "bCC")]
    b_cc: Option<Vec<ReplyTo>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReplyTo {
    #[serde(rename = "email")]
    email: Option<String>,

    #[serde(rename = "displayName")]
    display_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct EmailStatus {
    #[serde(rename = "messageId")]
    message_id: String,

    #[serde(rename = "status")]
    status: String,
}

#[derive(Debug)]
pub struct EndPointParams {
    host_name: String,
    access_key: String,
}

enum EmailStatusName {
    Unknown = 0,
    Queued = 1,
    OutForDelivery = 2,
    Dropped = 3,
}

impl fmt::Display for EmailStatusName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EmailStatusName::OutForDelivery => write!(f, "OutForDelivery"),
            EmailStatusName::Dropped => write!(f, "Dropped"),
            EmailStatusName::Queued => write!(f, "Queued"),
            EmailStatusName::Unknown => write!(f, ""),
        }
    }
}

impl FromStr for EmailStatusName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OutForDelivery" => Ok(EmailStatusName::OutForDelivery),
            "Dropped" => Ok(EmailStatusName::Dropped),
            "Queued" => Ok(EmailStatusName::Queued),
            _ => Ok(EmailStatusName::Unknown),
        }
    }
}

type HmacSha256 = Hmac<Sha256>;

fn compute_content_sha256(content: String) -> String {

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    return base64::encode(&result);
}

fn compute_signature(string_to_signed: String, secret: String) -> String {
    let mut mac = HmacSha256::new_from_slice(
        &base64::decode(secret).expect("HMAC compute decode secret failed"),
    )
    .expect("HMAC compuate_signature can take key of any size");

    mac.update(string_to_signed.as_bytes());

    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    return base64::encode(code_bytes);
}

async fn send_email(
    host_name: &String,
    access_key: &String,
    request_id: &String,
    request_email: &SentEmail,
) -> Result<String, Box<dyn std::error::Error>> {
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
        .await?;

    debug!("{:#?}", resp);
    let message_header = resp.headers().get("x-ms-request-id");
    let mut message_id = "";
    if let Some(hv) = message_header {
        message_id = hv.to_str().unwrap();
    }
    Ok(message_id.to_string())
}
//
//
//
async fn get_email_status(
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

fn parse_endpoint(endpoint: String) -> Result<EndPointParams, &'static str> {
    debug!("{}", "parse_endpoint");
    let parameters: Split<&str> = endpoint.split(";");
    if parameters.clone().count() != 2 {
        return Err("Connection String Invalid");
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
                    return Err("Endpoint invalid");
                }
                let host = param.substring(endpoint_str.len(), param.len());
                let host = Url::parse(host).unwrap().host().unwrap().to_string();
                endpoint.host_name = host;
                debug!("end point > {}", endpoint.host_name);
            }
            1 => {
                //get access key
                let accesskey_str = "accesskey=";
                if !param.starts_with(accesskey_str) {
                    return Err("Access Key invalid");
                }
                let access_key = param.substring(accesskey_str.len(), param.len());
                endpoint.access_key = access_key.to_string();
                debug!("access key > {}", endpoint.access_key);
            }
            _ => {}
        }
        idx = idx + 1;
    }
    Ok(endpoint)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let connection_str = env::var("CONNECTION_STR").unwrap();
    let sender = env::var("SENDER").unwrap();
    let reply_email_to = env::var("REPLY_EMAIL").unwrap();
    let reply_email_to_display = env::var("REPLY_EMAIL_DISPLAY").unwrap();

    let res_parse_endpoint = parse_endpoint(connection_str);
    if let Ok(endpoint) = res_parse_endpoint {
        let request_id = format!("{}", Uuid::new_v4());
        let access_key = endpoint.access_key;
        let host_name = endpoint.host_name;

        let email_request = SentEmail {
            headers: None,
            sender: Some(sender),
            content: Some(Content{
                subject : Some("An exciting offer especially for you!".to_string()),
                plain_text : Some("This exciting offer was created especially for you, our most loyal customer.".to_string()),
                html : Some("<html><head><title>Exciting offer!</title></head><body><h1>This exciting offer was created especially for you, our most loyal customer.</h1></body></html>".to_string())
            }),
            importance: Some("normal".to_string()),
            recipients: Some(Recipients{
                to: Some(vec![
                    ReplyTo{
                        email: Some(reply_email_to),
                        display_name : Some(reply_email_to_display)
                    },
                ]),
                cc: None,
                b_cc: None,
            }),
            attachments: None,
            reply_to: None,
            disable_user_engagement_tracking: Some(false),
        };

        let message_resp_id = send_email(
            &host_name.to_string(),
            &access_key.to_string(),
            &request_id,
            &email_request,
        )
        .await
        .unwrap();
        info!("email was sent with message id : {}", message_resp_id);
        loop {
            let status = get_email_status(
                &host_name.to_string(),
                &access_key.to_string(),
                &message_resp_id,
            )
            .await
            .unwrap();
            info!("get status of [{}] => {}", status.message_id, status.status);

            match EmailStatusName::from_str(status.status.as_str()).unwrap() {
                EmailStatusName::Queued => {
                    continue;
                }
                _ => {
                    break;
                }
            }
        }
        info!("========");
    } else {
        error!("{}", res_parse_endpoint.err().unwrap());
    }

    Ok(())
}
