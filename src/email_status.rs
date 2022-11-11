use log::debug;
use url::Url;
use crate::models::EmailStatus;
use crate::utils::{get_request_header};

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
    let client = reqwest::Client::new();

    let json_email_request = String::new(); //serde_json::to_string(&email_request).unwrap();
    let header = get_request_header(&url_endpoint,"GET",&request_id,&json_email_request,&access_key).unwrap();
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