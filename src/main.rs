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

/// Enum representing the authentication methods for the CLI.
#[derive(Debug, Clone, ValueEnum)]
pub enum CLIAuthenticationMethod {
    ManagedIdentity,
    ServicePrincipal,
    SharedKey,
}

/// Enum representing the protocols for the CLI.
#[derive(Debug, Clone, ValueEnum)]
pub enum CLIACSProtocol {
    REST,
    SMTP,
}

/// Struct representing the command line interface (CLI) arguments.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The protocol to use (REST or SMTP).
    #[arg(value_enum, short, long, default_value = "rest")]
    protocol: CLIACSProtocol,

    /// The authentication method to use.
    #[arg(value_enum, short, long, default_value = "shared-key")]
    auth_method: CLIAuthenticationMethod,
}

/// Sends an email using SMTP.
///
/// # Arguments
///
/// * `sender` - The sender's email address.
/// * `recipient` - The recipient's email address.
/// * `smtp_server` - The SMTP server address.
/// * `smtp_user` - The SMTP server username.
/// * `smtp_password` - The SMTP server password.
async fn send_email_with_smtp(
    sender: &str,
    recipient: &str,
    smtp_server: &str,
    smtp_user: &str,
    smtp_password: &str,
) {
    let email = Message::builder()
        .from(sender.parse().unwrap())
        .to(recipient.parse().unwrap())
        .subject("Happy new year")
        .header(ContentType::TEXT_PLAIN)
        .body(String::from("Be happy!"))
        .unwrap();

    debug!("Email: {:#?}", email);

    let creds = Credentials::new(smtp_user.to_owned(), smtp_password.to_owned());
    let mailer = SmtpTransport::starttls_relay(smtp_server)
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(r) => {
            debug!("Email sent: {:#?}", r);
            let messages = r.message();
            for message in messages {
                debug!("Message: {:#?}", message);
            }
            info!("Email sent successfully!")
        }
        Err(e) => error!("Could not send email: {e:?}"),
    }
}

/// Sends an email using the ACS client.
///
/// # Arguments
///
/// * `auth_method` - The authentication method to use.
/// * `sender` - The sender's email address.
/// * `recipient` - The recipient's email address.
/// * `display_name` - The display name for the recipient.
async fn send_email_with_api(
    auth_method: &CLIAuthenticationMethod,
    sender: &str,
    recipient: &str,
    display_name: &str,
) {
    let acs_client_builder: ACSClientBuilder = match auth_method {
        CLIAuthenticationMethod::ManagedIdentity => {
            info!("Using Managed Identity");
            let host_name = get_env_var("ASC_URL");
            debug!("host_name: {}", host_name);
            ACSClientBuilder::new()
                .managed_identity()
                .host(host_name.as_str())
        }
        CLIAuthenticationMethod::ServicePrincipal => {
            info!("Using Service Principal");
            let host_name = get_env_var("ASC_URL");
            let tenant_id = get_env_var("TENANT_ID");
            let client_id = get_env_var("CLIENT_ID");
            let client_secret = get_env_var("CLIENT_SECRET");
            debug!("host_name: {}", host_name);
            debug!("tenant_id: {}", tenant_id);
            debug!("client_id: {}", client_id);
            debug!("client_secret: {}", client_secret);
            ACSClientBuilder::new()
                .host(host_name.as_str())
                .service_principal(
                    tenant_id.as_str(),
                    client_id.as_str(),
                    client_secret.as_str(),
                )
        }
        CLIAuthenticationMethod::SharedKey => {
            info!("Using Shared Key");
            let connection_str = get_env_var("CONNECTION_STR");
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
                email: Some(recipient.to_owned()),
                display_name: Some(display_name.to_owned()),
            }]),
            cc: None,
            b_cc: None,
        })
        .user_engagement_tracking_disabled(false)
        .build()
        .expect("Failed to build SentEmail");

    debug!("Email request: {:#?}", email_request);

    let acs_client = acs_client_builder
        .build()
        .expect("Failed to build ACSClient");

    let resp_send_email = acs_client.send_email(&email_request).await;
    match resp_send_email {
        Ok(message_resp_id) => {
            info!("Email was sent with message id: {}", message_resp_id);
            loop {
                tokio::time::sleep(time::Duration::from_secs(5)).await;
                let resp_status = acs_client.get_email_status(&message_resp_id).await;
                if let Ok(status) = resp_status {
                    info!("{}\r\n", status.to_string());
                    if matches!(
                        status,
                        EmailSendStatusType::Unknown
                            | EmailSendStatusType::Canceled
                            | EmailSendStatusType::Failed
                            | EmailSendStatusType::Succeeded
                    ) {
                        break;
                    }
                } else {
                    error!("Error getting email status: {:?}", resp_status);
                    break;
                }
            }
        }
        Err(e) => error!("Error sending email: {:?}", e),
    }
}

/// Retrieves the value of an environment variable.
///
/// # Arguments
///
/// * `var_name` - The name of the environment variable.
///
/// # Returns
///
/// * `String` - The value of the environment variable.
fn get_env_var(var_name: &str) -> String {
    env::var(var_name).unwrap_or_else(|_| panic!("Environment variable {} is not set", var_name))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    dotenv::dotenv().ok();

    let args = Cli::parse();

    match args.protocol {
        CLIACSProtocol::REST => {
            info!("Sending email using REST API");
            let sender = get_env_var("SENDER");
            let recipient = get_env_var("REPLY_EMAIL");
            let display_name = get_env_var("REPLY_EMAIL_DISPLAY");

            send_email_with_api(
                &args.auth_method,
                sender.as_str(),
                recipient.as_str(),
                display_name.as_str(),
            )
                .await;
        }
        CLIACSProtocol::SMTP => {
            info!("Sending email using SMTP");
            let sender = get_env_var("SENDER");
            let recipient = get_env_var("REPLY_EMAIL");
            let smtp_server = get_env_var("SMTP_SERVER");
            let smtp_user = get_env_var("SMTP_USER");
            let smtp_password = get_env_var("SMTP_PASSWORD");

            send_email_with_smtp(
                sender.as_str(),
                recipient.as_str(),
                smtp_server.as_str(),
                smtp_user.as_str(),
                smtp_password.as_str(),
            )
                .await;
        }
    }

    Ok(())
}