use log::{debug, error};
use url::Url;
use crate::models::{CommunicationErrorResponse, SentEmail};
use crate::utils::{get_request_header};

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
    let client = reqwest::Client::new();

    let json_email_request = serde_json::to_string(request_email).unwrap();
    let header = get_request_header(&url_endpoint,"POST",&request_id,&json_email_request,&access_key).unwrap();

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
                return Err(body.error.message);
            }else{
                return Err(error_reponse.err().unwrap().to_string())
            }
        }
    }else{
        return Err(resp.err().unwrap().to_string());
    }
}