use std::env;
use std::str::FromStr;
use log::{error, info};
use uuid::Uuid;
use crate::email_status::get_email_status;
use crate::models::{Content, EmailStatusName, Recipients, ReplyTo, SentEmail};
use crate::send_email::send_email;
use crate::utils::parse_endpoint;

mod models;
mod utils;
mod send_email;
mod email_status;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let connection_str = env::var("CONNECTION_STR").unwrap();
    let sender = env::var("SENDER").unwrap();
    let reply_email_to = env::var("REPLY_EMAIL").unwrap();
    let reply_email_to_display = env::var("REPLY_EMAIL_DISPLAY").unwrap();

    let res_parse_endpoint = parse_endpoint(&connection_str);
    if let Ok(endpoint) = res_parse_endpoint {
        let request_id = format!("{}", Uuid::new_v4());
        let access_key = endpoint.access_key;
        let host_name = endpoint.host_name;

        let email_request = SentEmail {
            headers: None,
            sender: Some(sender),
            content: Some(Content {
                subject: Some("An exciting offer especially for you!".to_string()),
                plain_text: Some("This exciting offer was created especially for you, our most loyal customer.".to_string()),
                html: Some("<html><head><title>Exciting offer!</title></head><body><h1>This exciting offer was created especially for you, our most loyal customer.</h1></body></html>".to_string())
            }),
            importance: Some("normal".to_string()),
            recipients: Some(Recipients {
                to: Some(vec![
                    ReplyTo {
                        email: Some(reply_email_to),
                        display_name: Some(reply_email_to_display)
                    },
                ]),
                cc: None,
                b_cc: None,
            }),
            attachments: None,
            reply_to: None,
            disable_user_engagement_tracking: Some(false),
        };
        let resp_send_email = send_email(
            &host_name.to_string(),
            &access_key.to_string(),
            &request_id,
            &email_request,
        ).await;
        if let Ok(message_resp_id) = resp_send_email {
            info!("email was sent with message id : {}", message_resp_id);
            loop {
                let resp_status = get_email_status(
                    &host_name.to_string(),
                    &access_key.to_string(),
                    &message_resp_id,
                ).await;
                if let Ok(status) = resp_status {
                    info!("get status of [{}] => {}", status.message_id, status.status);
                    match EmailStatusName::from_str(status.status.as_str()).unwrap() {
                        EmailStatusName::Queued => {
                            continue;
                        }
                        _ => {
                            break;
                        }
                    }
                }else{
                    error!("{}",  resp_status.err().unwrap());
                    break;
                }
            }
            info!("========");
        }else{
            error!("{}",resp_send_email.err().unwrap());
        }
    }
    Ok(())
}
