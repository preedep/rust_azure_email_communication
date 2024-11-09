use crate::email::{get_email_status, send_email};
use crate::models::{
    EmailAddress, EmailContent, EmailSendStatusType, Recipients, SentEmailBuilder,
};
use crate::utils::parse_endpoint;
use log::{debug, error, info};
use std::{env, time};
use uuid::Uuid;

mod email;
mod models;
mod utils;

use clap::{Parser, ValueEnum};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

#[derive(Debug, Clone, ValueEnum)]
enum ACSProtocol {
    REST,
    SMTP,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The protocol to use for sending the email (REST or SMTP) (default: REST)
    #[arg(value_enum, short, long, default_value = "REST")]
    protocol: ACSProtocol,
}
/// Send email using SMTP
/// This function sends an email using SMTP
/// The sender, reply email, smtp server, smtp user and smtp password are read from the environment variables
async fn send_email_with_smtp(
    sender: &str,
    reply_email_to: &str,
    smtp_server: &str,
    smtp_user: &str,
    smtp_password: &str,
) {
    let email = Message::builder()
        .from(sender.parse().unwrap())
        //.reply_to(reply_email_to.parse().unwrap())
        .to(reply_email_to.parse().unwrap())
        .subject("Happy new year")
        .header(ContentType::TEXT_PLAIN)
        .body(String::from("Be happy!"))
        .unwrap();

    debug!("Email: {:#?}", email);

    let creds = Credentials::new(smtp_user.to_owned(), smtp_password.to_owned());

    // Open a remote connection to gmail

    let mailer = SmtpTransport::starttls_relay(smtp_server)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => info!("Email sent successfully!"),
        Err(e) => error!("Could not send email: {e:?}"),
    }
}
/// Send email using REST API
/// This function sends an email using the REST API
/// The sender, reply email, connection string, access key and host name are read from the environment variables
async fn send_email_with_api(
    connection_str: &str,
    sender: &str,
    reply_email_to: &str,
    reply_email_to_display: &str,
) {
    let res_parse_endpoint = parse_endpoint(&connection_str);
    if let Ok(endpoint) = res_parse_endpoint {
        let request_id = format!("{}", Uuid::new_v4());
        let access_key = endpoint.access_key;
        let host_name = endpoint.host_name;

        let email_request = SentEmailBuilder::new()
            .sender(sender.to_owned())
            .content(EmailContent {
                subject: Some("An exciting offer especially for you!".to_string()),
                plain_text: Some("This exciting offer was created especially for you, our most loyal customer.".to_string()),
                html: Some("<html><head><title>Exciting offer!</title></head><body><h1>This exciting offer was created especially for you, our most loyal customer.</h1></body></html>".to_string()),
            })
            .recipients(Recipients {
                to: Some(vec![EmailAddress {
                    email: Some(reply_email_to.to_owned()),
                    display_name: Some(reply_email_to_display.to_owned()),
                }]),
                cc: None,
                b_cc: None,
            })
            .user_engagement_tracking_disabled(false)
            .build()
            .expect("Failed to build SentEmail");

        debug!("Email request: {:#?}", email_request);

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
                    tokio::time::sleep(time::Duration::from_secs(5)).await;
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
    } else {
        error!("Error parsing endpoint: {:?}", res_parse_endpoint);
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    dotenv::dotenv().ok();

    let args = Cli::parse();
    match args.protocol {
        ACSProtocol::REST => {
            info!("Sending email using REST API");

            let connection_str = env::var("CONNECTION_STR").unwrap();
            let sender = env::var("SENDER").unwrap();
            let reply_email_to = env::var("REPLY_EMAIL").unwrap();
            let reply_email_to_display = env::var("REPLY_EMAIL_DISPLAY").unwrap();

            send_email_with_api(
                connection_str.as_str(),
                sender.as_str(),
                reply_email_to.as_str(),
                reply_email_to_display.as_str(),
            )
            .await;
        }
        ACSProtocol::SMTP => {
            info!("Sending email using SMTP");
            let sender = env::var("SENDER").unwrap();
            let reply_email_to = env::var("REPLY_EMAIL").unwrap();

            let smtp_server = env::var("SMTP_SERVER").unwrap();
            let smtp_user = env::var("SMTP_USER").unwrap();
            let smtp_password = env::var("SMTP_PASSWORD").unwrap();

            send_email_with_smtp(
                sender.as_str(),
                reply_email_to.as_str(),
                smtp_server.as_str(),
                smtp_user.as_str(),
                smtp_password.as_str(),
            )
            .await;
        }
    }

    Ok(())
}
