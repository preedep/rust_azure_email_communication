use crate::models::{SentEmailResponse, SentEmail, ErrorDetail, EmailSendStatusType};
use crate::utils::get_request_header;
use reqwest::StatusCode;
use url::Url;


type EmailResult<T> = std::result::Result<T, ErrorDetail>;


pub async fn get_email_status(
    host_name: &String,
    access_key: &String,
    request_id: &String,
) -> EmailResult<EmailSendStatusType> {
    let url = format!(
        "https://{}/emails/operations/{}?api-version=2023-01-15-preview",
        host_name, request_id,
    );
    //debug!("{}", url);
    let url_endpoint = Url::parse(url.as_str()).unwrap();
    //debug!("{:#?}", url_endpoint);
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
    let resp = client.get(url).headers(header).send().await;

    return if let Ok(resp) = resp {
        //debug!("{:#?}", resp);
        if resp.status() == StatusCode::OK {
            let email_resp = resp.json::<SentEmailResponse>().await.expect("Response Invalid");
            Ok(email_resp.status.unwrap().to_type())
        }else{
            let email_resp = resp.json::<ErrorDetail>().await.expect("Response Invalid");
            Err(email_resp)
        }
    } else {
        Err(ErrorDetail{
            additional_info: None,
            code: None,
            message: Some(resp.err().unwrap().to_string()),
            target: None,
        })
    };
}

pub async fn send_email(
    host_name: &String,
    access_key: &String,
    request_id: &String,
    request_email: &SentEmail,
) -> EmailResult<String> {
    let url = format!(
        "https://{}/emails:send?api-version=2023-01-15-preview",
        host_name
    );
    //debug!("{}", url);
    let url_endpoint = Url::parse(url.as_str()).unwrap();
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
        //debug!("{:#?}", resp);
        if resp.status() == StatusCode::ACCEPTED {
            let email_resp = resp.json::<SentEmailResponse>().await.expect("Response Invalid");
            Ok(email_resp.id.unwrap_or("0".to_string()))
        }else{
            let email_resp = resp.json::<ErrorDetail>().await.expect("Response Invalid");
            Err(email_resp)
        }
    } else {
        Err(ErrorDetail{
            additional_info: None,
            code: None,
            message: Some(resp.err().unwrap().to_string()),
            target: None,
        })
    };
}
