use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

/// Represents the status of an email send operation.
#[derive(Serialize, Deserialize, Debug)]
pub struct EmailSendStatus(EmailSendStatusType);

impl EmailSendStatus {
    /// Converts the `EmailSendStatus` to its underlying type.
    ///
    /// # Returns
    ///
    /// * `EmailSendStatusType` - The underlying type of the email send status.
    pub fn to_type(self) -> EmailSendStatusType {
        self.0
    }
}

/// Enum representing the possible statuses of an email send operation.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum EmailSendStatusType {
    Unknown,
    Canceled,
    Failed,
    NotStarted,
    Running,
    Succeeded,
}

/// Represents the response received after sending an email.
#[derive(Serialize, Deserialize, Debug)]
pub struct SentEmailResponse {
    /// The ID of the sent email.
    #[serde(rename = "id")]
    pub id: Option<String>,

    /// The status of the sent email.
    #[serde(rename = "status")]
    pub status: Option<EmailSendStatus>,

    /// The error details if the email send operation failed.
    #[serde(rename = "error")]
    pub error: Option<ErrorDetail>,
}

/// Represents the details of an error.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ErrorDetail {
    /// Additional information about the error.
    #[serde(rename = "additionalInfo")]
    pub additional_info: Option<Vec<ErrorAdditionalInfo>>,

    /// The error code.
    #[serde(rename = "code")]
    pub code: Option<String>,

    // #[serde(rename = "details")]
    // pub(crate) details: Option<ErrorDetail>,
    /// The error message.
    #[serde(rename = "message")]
    pub message: Option<String>,

    /// The target of the error.
    #[serde(rename = "target")]
    pub target: Option<String>,
}

/// Represents additional information about an error.
#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorAdditionalInfo {
    /// The additional information.
    #[serde(rename = "info")]
    pub info: Option<String>,

    /// The type of the additional information.
    #[serde(rename = "type")]
    pub info_type: Option<String>,
}

/// Represents an email to be sent.
#[derive(Serialize, Deserialize, Debug)]
pub struct SentEmail {
    /// The headers of the email.
    #[serde(rename = "headers", skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<Header>>,

    /// The sender address of the email.
    #[serde(rename = "senderAddress")]
    pub sender: String,

    /// The content of the email.
    #[serde(rename = "content")]
    pub content: EmailContent,

    /// The recipients of the email.
    #[serde(rename = "recipients")]
    pub recipients: Recipients,

    /// The attachments of the email.
    #[serde(rename = "attachments", skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<EmailAttachment>>,

    /// The reply-to addresses of the email.
    #[serde(rename = "replyTo", skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<Vec<EmailAddress>>,

    /// Indicates whether user engagement tracking is disabled.
    #[serde(
        rename = "userEngagementTrackingDisabled",
        skip_serializing_if = "Option::is_none"
    )]
    pub user_engagement_tracking_disabled: Option<bool>,
}

/// Builder for creating a `SentEmail` instance.
pub struct SentEmailBuilder {
    headers: Option<Vec<Header>>,
    sender: Option<String>,
    content: Option<EmailContent>,
    recipients: Option<Recipients>,
    attachments: Option<Vec<EmailAttachment>>,
    reply_to: Option<Vec<EmailAddress>>,
    user_engagement_tracking_disabled: Option<bool>,
}

impl SentEmailBuilder {
    /// Creates a new `SentEmailBuilder` instance.
    ///
    /// # Returns
    ///
    /// * `SentEmailBuilder` - A new instance of the builder.
    pub fn new() -> Self {
        SentEmailBuilder {
            headers: None,
            sender: None,
            content: None,
            recipients: None,
            attachments: None,
            reply_to: None,
            user_engagement_tracking_disabled: None,
        }
    }

    /// Sets the headers for the email.
    ///
    /// # Arguments
    ///
    /// * `headers` - A vector of `Header` instances.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder instance.
    #[allow(dead_code)]
    pub fn headers(mut self, headers: Vec<Header>) -> Self {
        self.headers = Some(headers);
        self
    }

    /// Sets the sender address for the email.
    ///
    /// # Arguments
    ///
    /// * `sender` - A string representing the sender address.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder instance.
    pub fn sender(mut self, sender: String) -> Self {
        self.sender = Some(sender);
        self
    }

    /// Sets the content for the email.
    ///
    /// # Arguments
    ///
    /// * `content` - An `EmailContent` instance.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder instance.
    pub fn content(mut self, content: EmailContent) -> Self {
        self.content = Some(content);
        self
    }

    /// Sets the recipients for the email.
    ///
    /// # Arguments
    ///
    /// * `recipients` - A `Recipients` instance.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder instance.
    pub fn recipients(mut self, recipients: Recipients) -> Self {
        self.recipients = Some(recipients);
        self
    }

    /// Sets the attachments for the email.
    ///
    /// # Arguments
    ///
    /// * `attachments` - A vector of `EmailAttachment` instances.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder instance.
    #[allow(dead_code)]
    pub fn attachments(mut self, attachments: Vec<EmailAttachment>) -> Self {
        self.attachments = Some(attachments);
        self
    }

    /// Sets the reply-to addresses for the email.
    ///
    /// # Arguments
    ///
    /// * `reply_to` - A vector of `EmailAddress` instances.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder instance.
    #[allow(dead_code)]
    pub fn reply_to(mut self, reply_to: Vec<EmailAddress>) -> Self {
        self.reply_to = Some(reply_to);
        self
    }

    /// Sets whether user engagement tracking is disabled for the email.
    ///
    /// # Arguments
    ///
    /// * `user_engagement_tracking_disabled` - A boolean indicating whether tracking is disabled.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder instance.
    pub fn user_engagement_tracking_disabled(
        mut self,
        user_engagement_tracking_disabled: bool,
    ) -> Self {
        self.user_engagement_tracking_disabled = Some(user_engagement_tracking_disabled);
        self
    }

    /// Builds the `SentEmail` instance.
    ///
    /// # Returns
    ///
    /// * `Result<SentEmail, &\`static str\>` - The built `SentEmail` instance or an error message.
    pub fn build(self) -> Result<SentEmail, &'static str> {
        Ok(SentEmail {
            headers: self.headers,
            sender: self.sender.ok_or("Sender is required")?,
            content: self.content.ok_or("Content is required")?,
            recipients: self.recipients.ok_or("Recipients are required")?,
            attachments: self.attachments,
            reply_to: self.reply_to,
            user_engagement_tracking_disabled: self.user_engagement_tracking_disabled,
        })
    }
}

/// Represents an email attachment.
#[derive(Serialize, Deserialize, Debug)]
pub struct EmailAttachment {
    /// The name of the attachment.
    #[serde(rename = "name")]
    name: Option<String>,

    /// The content type of the attachment.
    #[serde(rename = "contentType")]
    attachment_type: Option<String>,

    /// The base64 encoded content of the attachment.
    #[serde(rename = "contentInBase64")]
    content_bytes_base64: Option<String>,
}

/// Represents the content of an email.
#[derive(Serialize, Deserialize, Debug)]
pub struct EmailContent {
    /// The subject of the email.
    #[serde(rename = "subject")]
    pub subject: Option<String>,

    /// The plain text content of the email.
    #[serde(rename = "plainText")]
    pub plain_text: Option<String>,

    /// The HTML content of the email.
    #[serde(rename = "html")]
    pub html: Option<String>,
}

/// Represents a header in an email.
#[derive(Serialize, Deserialize, Debug)]
pub struct Header {
    /// The name of the header.
    #[serde(rename = "name")]
    name: Option<String>,

    /// The value of the header.
    #[serde(rename = "value")]
    value: Option<String>,
}

/// Represents the recipients of an email.
#[derive(Serialize, Deserialize, Debug)]
pub struct Recipients {
    /// The primary recipients of the email.
    #[serde(rename = "to")]
    pub to: Option<Vec<EmailAddress>>,

    /// The CC recipients of the email.
    #[serde(rename = "cc")]
    pub cc: Option<Vec<EmailAddress>>,

    /// The BCC recipients of the email.
    #[serde(rename = "bcc")]
    pub b_cc: Option<Vec<EmailAddress>>,
}

/// Represents an email address.
#[derive(Serialize, Deserialize, Debug)]
pub struct EmailAddress {
    /// The email address.
    #[serde(rename = "address")]
    pub email: Option<String>,

    /// The display name associated with the email address.
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

/// Represents an error response.
#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    /// The error details.
    #[serde(rename = "error")]
    pub error: Option<ErrorDetail>,
}

/// Represents the parameters of an endpoint.
#[derive(Debug)]
pub struct EndPointParams {
    /// The host name of the endpoint.
    pub host_name: String,

    /// The access key for the endpoint.
    pub access_key: String,
}

impl fmt::Display for EmailSendStatusType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EmailSendStatusType::Canceled => write!(f, "Canceled"),
            EmailSendStatusType::Failed => write!(f, "Failed"),
            EmailSendStatusType::NotStarted => write!(f, "NotStarted"),
            EmailSendStatusType::Running => write!(f, "Running"),
            EmailSendStatusType::Succeeded => write!(f, "Succeeded"),
            _ => write!(f, "Unknown"),
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
        write!(f, "{}", self.0).expect("EmailSendStatus: panic message");
        Ok(())
    }
}
