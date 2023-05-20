use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Formatter};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
pub struct  EmailSendStatus(EmailSendStatusType);
impl EmailSendStatus {
    pub fn to_type(self) -> EmailSendStatusType {
        self.0
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub enum EmailSendStatusType {
    Unknown,
    Canceled,
    Failed,
    NotStarted,
    Running,
    Succeeded
}
#[derive(Serialize, Deserialize, Debug)]
pub struct SentEmailResponse {
    #[serde(rename = "id")]
    pub(crate) id: Option<String>,

    #[serde(rename = "status")]
    pub(crate) status: Option<EmailSendStatus>,

    #[serde(rename = "error")]
    pub(crate) error: Option<ErrorDetail>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorDetail {
    #[serde(rename = "additionalInfo")]
    pub(crate) additional_info: Option<Vec<ErrorAdditionalInfo>>,

     #[serde(rename = "code")]
    pub(crate) code: Option<String>,

   // #[serde(rename = "details")]
   // pub(crate) details: Option<ErrorDetail>,

    #[serde(rename = "message")]
    pub(crate) message: Option<String>,

    #[serde(rename = "target")]
    pub(crate) target: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorAdditionalInfo {
    #[serde(rename = "info")]
    pub(crate) info: Option<String>,

    #[serde(rename = "type")]
    pub(crate) info_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SentEmail {
    #[serde(rename = "headers")]
    pub(crate) headers: Option<Vec<Header>>,

    #[serde(rename = "senderAddress")]
    pub(crate) sender: Option<String>,

    #[serde(rename = "content")]
    pub(crate) content: Option<EmailContent>,

    #[serde(rename = "recipients")]
    pub(crate) recipients: Option<Recipients>,

    #[serde(rename = "attachments")]
    pub(crate) attachments: Option<Vec<EmailAttachment>>,

    #[serde(rename = "replyTo")]
    pub(crate) reply_to: Option<Vec<EmailAddress>>,

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


#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    #[serde(rename = "error")]
    pub(crate) error: Option<ErrorDetail>,
}

#[derive(Debug)]
pub struct EndPointParams {
    pub(crate) host_name: String,
    pub(crate) access_key: String,
}

impl fmt::Display for EmailSendStatusType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EmailSendStatusType::Canceled => write!(f, "Canceled"),
            EmailSendStatusType::Failed => write!(f, "Failed"),
            EmailSendStatusType::NotStarted => write!(f, "NotStarted"),
            EmailSendStatusType::Running => write!(f, "Running"),
            EmailSendStatusType::Succeeded => write!(f,"Succeeded"),
            _ => write!(f,"Unknown")
        }
    }
}

impl FromStr for EmailSendStatusType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Canceled" => Ok(EmailSendStatusType::Canceled),
            "Failed" => Ok(EmailSendStatusType::Failed),
            "NotStarted" => Ok(EmailSendStatusType::NotStarted),
            "Running" => Ok(EmailSendStatusType::Running),
            "Succeeded" => Ok(EmailSendStatusType::Succeeded),
            _ => Ok(EmailSendStatusType::Unknown),
        }
    }
}

impl fmt::Display for EmailSendStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.0).expect("EmailSendStatus: panic message");
        Ok(())
    }
}
