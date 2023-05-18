use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
pub struct SentEmailResponse {
    #[serde(rename = "id")]
    pub(crate) id: Option<String>,

    #[serde(rename = "status")]
    pub(crate) status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SentEmail {
    #[serde(rename = "headers")]
    pub(crate) headers: Option<Vec<Header>>,

    #[serde(rename = "senderAddress")]
    pub(crate) sender: Option<String>,

    #[serde(rename = "content")]
    pub(crate) content: Option<EmailContent>,

    /*
        #[serde(rename = "importance")]
        pub(crate) importance: Option<String>,
    */
    #[serde(rename = "recipients")]
    pub(crate) recipients: Option<Recipients>,

    #[serde(rename = "attachments")]
    pub(crate) attachments: Option<Vec<EmailAttachment>>,

    #[serde(rename = "replyTo")]
    pub(crate) reply_to: Option<Vec<EmailAddress>>,

    /*
        #[serde(rename = "disableUserEngagementTracking")]
        pub(crate) disable_user_engagement_tracking: Option<bool>,
    */
    #[serde(rename = "userEngagementTrackingDisabled")]
    pub(crate) user_engagement_tracking_disabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailAttachment {
    #[serde(rename = "name")]
    name: Option<String>,

    #[serde(rename = "contentType")]
    attachment_type: Option<String>,

    #[serde(rename = "contentInBase64")]
    content_bytes_base64: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailContent {
    #[serde(rename = "subject")]
    pub(crate) subject: Option<String>,

    #[serde(rename = "plainText")]
    pub(crate) plain_text: Option<String>,

    #[serde(rename = "html")]
    pub(crate) html: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Header {
    #[serde(rename = "name")]
    name: Option<String>,

    #[serde(rename = "value")]
    value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Recipients {
    #[serde(rename = "to")]
    pub(crate) to: Option<Vec<EmailAddress>>,

    #[serde(rename = "cc")]
    pub(crate) cc: Option<Vec<EmailAddress>>,

    #[serde(rename = "bcc")]
    pub(crate) b_cc: Option<Vec<EmailAddress>>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct EmailAddress {
    #[serde(rename = "address")]
    pub(crate) email: Option<String>,

    #[serde(rename = "displayName")]
    pub(crate) display_name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct EmailStatus {
    #[serde(rename = "id")]
    pub(crate) message_id: String,

    #[serde(rename = "status")]
    pub(crate) status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommunicationError {
    #[serde(rename = "code")]
    code: String,

    #[serde(rename = "message")]
    pub(crate) message: String,

    #[serde(rename = "target")]
    target: Option<String>,

    #[serde(rename = "details")]
    details: Option<Vec<Box<CommunicationError>>>,

    #[serde(rename = "innererror")]
    innererror: Option<Box<CommunicationError>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommunicationErrorResponse {
    #[serde(rename = "error")]
    pub(crate) error: CommunicationError,
}

#[derive(Debug)]
pub struct EndPointParams {
    pub(crate) host_name: String,
    pub(crate) access_key: String,
}

pub enum EmailStatusName {
    Unknown = 0,
    Queued = 1,
    OutForDelivery = 2,
    Dropped = 3,
}

impl fmt::Display for EmailStatusName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EmailStatusName::OutForDelivery => write!(f, "OutForDelivery"),
            EmailStatusName::Dropped => write!(f, "Dropped"),
            EmailStatusName::Queued => write!(f, "Queued"),
            EmailStatusName::Unknown => write!(f, ""),
        }
    }
}

impl FromStr for EmailStatusName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OutForDelivery" => Ok(EmailStatusName::OutForDelivery),
            "Dropped" => Ok(EmailStatusName::Dropped),
            "Queued" => Ok(EmailStatusName::Queued),
            _ => Ok(EmailStatusName::Unknown),
        }
    }
}
