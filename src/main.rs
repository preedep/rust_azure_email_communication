use crate::models::{
    EmailAddress, EmailContent, EmailSendStatusType, Recipients, SentEmailBuilder,
};
use log::{debug, error, info};
use std::{env, time};
mod acs_email;
mod models;
mod utils;

use crate::acs_email::ACSClientBuilder;
use clap::{Parser, ValueEnum};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

// Define the AuthenticationMethod enum
#[derive(Debug, Clone, ValueEnum)]
pub enum CLIAuthenticationMethod {
    ManagedIdentity,
    ServicePrincipal,
    SharedKey,
}

// Define the ACSProtocol enum
#[derive(Debug, Clone, ValueEnum)]
pub enum CLIACSProtocol {
    REST,
    SMTP,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The protocol to use for sending the email (REST: value = rest or SMTP: value = smtp)
    #[arg(value_enum, short, long, default_value = "rest")]
    protocol: CLIACSProtocol,

    /// The authentication method to use (ManagedIdentity: value = managed-identity , ServicePrincipal: value = service-principal, SharedKey: value = shared-key)
    #[arg(value_enum, short, long, default_value = "shared-key")]
    auth_method: CLIAuthenticationMethod,
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
    auth_method: &CLIAuthenticationMethod,
    sender: &str,
    reply_email_to: &str,
    reply_email_to_display: &str,
) {

    let acs_client_builder : ACSClientBuilder= match auth_method {
        CLIAuthenticationMethod::ManagedIdentity => {
            info!("Using Managed Identity");
            let host_name = env::var("ASC_URL").unwrap();
            debug!("host_name: {}", host_name);
            ACSClientBuilder::new().managed_identity().host(host_name.as_str())
        }
        CLIAuthenticationMethod::ServicePrincipal => {
            info!("Using Service Principal");
            let host_name = env::var("ASC_URL").unwrap();
            let tenant_id = env::var("TENANT_ID").unwrap();
            let client_id = env::var("CLIENT_ID").unwrap();
            let client_secret = env::var("CLIENT_SECRET").unwrap();

            debug!("host_name: {}", host_name);
            debug!("tenant_id: {}", tenant_id);
            debug!("client_id: {}", client_id);
            debug!("client_secret: {}", client_secret);

            ACSClientBuilder::new()
                .host(host_name.as_str())
                .service_principal(tenant_id.as_str(), client_id.as_str(), client_secret.as_str())
        }
        CLIAuthenticationMethod::SharedKey => {
            info!("Using Shared Key");
            let connection_str = env::var("CONNECTION_STR").unwrap();
            ACSClientBuilder::new().connection_string(connection_str.as_str())
        }
    };

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


    let acs_client = acs_client_builder.build().expect("Failed to build ACSClient");

    let resp_send_email = acs_client.send_email(&email_request).await;

    match resp_send_email {
        Ok(message_resp_id) => {
            info!("email was sent with message id : {}", message_resp_id);
            loop {
                tokio::time::sleep(time::Duration::from_secs(5)).await;
                let resp_status = acs_client.get_email_status(&message_resp_id).await;
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
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    dotenv::dotenv().ok();

    let args = Cli::parse();
    match args.protocol {
        CLIACSProtocol::REST => {
            info!("Sending email using REST API");


            let sender = env::var("SENDER").unwrap();
            let reply_email_to = env::var("REPLY_EMAIL").unwrap();
            let reply_email_to_display = env::var("REPLY_EMAIL_DISPLAY").unwrap();


            send_email_with_api(
                &args.auth_method,
                sender.as_str(),
                reply_email_to.as_str(),
                reply_email_to_display.as_str(),
            )
            .await;
        }
        CLIACSProtocol::SMTP => {
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
