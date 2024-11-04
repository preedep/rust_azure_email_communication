use crate::email::{get_email_status, send_email};
use crate::models::{EmailAddress, EmailContent, EmailSendStatusType, Recipients, SentEmail};
use crate::utils::parse_endpoint;
use log::{error, info};
use std::thread::sleep;
use std::{env, time};
use uuid::Uuid;

mod email;
mod models;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    dotenv::dotenv().ok();

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
            content: Some(EmailContent {
                subject: Some("An exciting offer especially for you!".to_string()),
                plain_text: Some("This exciting offer was created especially for you, our most loyal customer.".to_string()),
                html: Some("<html><head><title>Exciting offer!</title></head><body><h1>This exciting offer was created especially for you, our most loyal customer.</h1></body></html>".to_string())
            }),

            recipients: Some(Recipients {
                to: Some(vec![
                    EmailAddress {
                        email: Some(reply_email_to),
                        display_name: Some(reply_email_to_display)
                    },
                ]),
                cc: None,
                b_cc: None,
            }),
            attachments: None,
            reply_to: None,
            //disable_user_engagement_tracking: Some(false),
            user_engagement_tracking_disabled: Some(false),
        };
        let resp_send_email = send_email(
            &host_name.to_string(),
            &access_key.to_string(),
            &request_id,
            &email_request,
        )
        .await;

        match resp_send_email {
            Ok(message_resp_id) => {
                info!("email was sent with message id : {}", message_resp_id);
                loop {
                    tokio::time::sleep(time::Duration::from_secs(1)).await;

                    let resp_status = get_email_status(
                        &host_name.to_string(),
                        &access_key.to_string(),
                        &message_resp_id,
                    )
                    .await;
                    if let Ok(status) = resp_status {
                        //let status = status.status.unwrap();
                        info!("{}\r\n", status.to_string());
                        match status {
                            EmailSendStatusType::Unknown => {
                                break;
                            }
                            EmailSendStatusType::Canceled => {
                                break;
                            }
                            EmailSendStatusType::Failed => {
                                break;
                            }
                            EmailSendStatusType::NotStarted => {}
                            EmailSendStatusType::Running => {}
                            EmailSendStatusType::Succeeded => {
                                break;
                            }
                        }
                    } else {
                        error!("Error getting email status: {:?}", resp_status);
                        break;
                    }
                }
                info!("========");
            }
            Err(e) => {
                error!("Error sending email: {:?}", e);
            }
        }
    }
    Ok(())
}
