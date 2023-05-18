use crate::models::{CommunicationErrorResponse, EmailStatus, SentEmail, SentEmailResponse};
use crate::utils::get_request_header;
use log::{debug};
use url::Url;

pub async fn get_email_status(
    host_name: &String,
    access_key: &String,
    request_id: &String,
) -> Result<EmailStatus, Box<dyn std::error::Error>> {

    let url = format!(
        "https://{}/emails/operations/{}/status?api-version=2023-01-15-preview",
        host_name, request_id,
    );

    debug!("{}", url);
    let url_endpoint = Url::parse(url.as_str()).unwrap();
    debug!("{:#?}", url_endpoint);
    /*
     cal HMAC-SHA256
    */
    let client = reqwest::Client::new();

    let json_email_request = String::new(); //serde_json::to_string(&email_request).unwrap();
    let header = get_request_header(
        &url_endpoint,
        "GET",
        &request_id,
        &json_email_request,
        &access_key,
    )
    .unwrap();
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

pub async fn send_email(
    host_name: &String,
    access_key: &String,
    request_id: &String,
    request_email: &SentEmail,
) -> Result<String, String> {

    let url = format!(
        "https://{}/emails:send?api-version=2023-01-15-preview",
        host_name
    );
    //debug!("{}", url);
    let url_endpoint = Url::parse(url.as_str()).unwrap();
    debug!("{:#?}", url_endpoint);
    debug!("{:#?}", request_email);
    /*
     cal HMAC-SHA256
    */
    let client = reqwest::Client::new();

    let json_email_request = serde_json::to_string(request_email).unwrap();
    let header = get_request_header(
        &url_endpoint,
        "POST",
        &request_id,
        &json_email_request,
        &access_key,
    )
    .unwrap();

    let resp = client
        .post(url)
        .headers(header)
        .json(request_email)
        .send()
        .await;

    return if let Ok(resp) = resp {
        if resp.status().is_success() {
            debug!("{:#?}", resp);
            let email_resp = resp.json::<SentEmailResponse>().await;
            if let Ok(resp) = email_resp {
                Ok(resp.id.unwrap_or("".to_string()))
            } else {
                Err(email_resp.err().unwrap().to_string())
            }
        } else {
            let error_response = resp.json::<CommunicationErrorResponse>().await;
            if let Ok(body) = error_response {
                Err(body.error.message)
            } else {
                Err(error_response.err().unwrap().to_string())
            }
        }
    } else {
        Err(resp.err().unwrap().to_string())
    };
}
