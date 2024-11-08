use crate::email::{get_email_status, send_email};
use crate::models::{
    EmailAddress, EmailContent, EmailSendStatusType, Recipients,  SentEmailBuilder,
};
use crate::utils::parse_endpoint;
use log::{debug, error, info};
use std::{env, time};
use uuid::Uuid;

mod email;
mod models;
mod utils;

use clap::{Parser, Subcommand, ValueEnum};
use lettre::{Message, SmtpTransport, Transport};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;

#[derive(Debug,Clone,ValueEnum)]
enum ACSProtocol {
    REST,
    SMTP
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
async fn send_email_with_smtp(){

    let sender = env::var("SENDER").unwrap();
    let reply_email_to = env::var("REPLY_EMAIL").unwrap();

    let smtp_server = env::var("SMTP_SERVER").unwrap();
    let smtp_user = env::var("SMTP_USER").unwrap();
    let smtp_password = env::var("SMTP_PASSWORD").unwrap();

    let email = Message::builder()
        .from(sender.parse().unwrap())
        //.reply_to(reply_email_to.parse().unwrap())
        .to(reply_email_to.parse().unwrap())
        .subject("Happy new year")
        .header(ContentType::TEXT_PLAIN)
        .body(String::from("Be happy!"))
        .unwrap();


    debug!("Email: {:#?}", email);

    let creds = Credentials::new(smtp_user, smtp_password);

    // Open a remote connection to gmail

    let mailer = SmtpTransport::starttls_relay(smtp_server.as_str())
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
async fn send_email_with_api(){
    let connection_str = env::var("CONNECTION_STR").unwrap();
    let sender = env::var("SENDER").unwrap();
    let reply_email_to = env::var("REPLY_EMAIL").unwrap();
    let reply_email_to_display = env::var("REPLY_EMAIL_DISPLAY").unwrap();

    let res_parse_endpoint = parse_endpoint(&connection_str);
    if let Ok(endpoint) = res_parse_endpoint {
        let request_id = format!("{}", Uuid::new_v4());
        let access_key = endpoint.access_key;
        let host_name = endpoint.host_name;

        let email_request = SentEmailBuilder::new()
            .sender(sender)
            .content(EmailContent {
                subject: Some("An exciting offer especially for you!".to_string()),
                plain_text: Some("This exciting offer was created especially for you, our most loyal customer.".to_string()),
                html: Some("<html><head><title>Exciting offer!</title></head><body><h1>This exciting offer was created especially for you, our most loyal customer.</h1></body></html>".to_string()),
            })
            .recipients(Recipients {
                to: Some(vec![EmailAddress {
                    email: Some(reply_email_to),
                    display_name: Some(reply_email_to_display),
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
            send_email_with_api().await;
        }
        ACSProtocol::SMTP => {
            info!("Sending email using SMTP");
            send_email_with_smtp().await;
        }
    }


    Ok(())
}
