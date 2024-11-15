# Azure Email Communication Service - Rust


## I've change repo to https://github.com/preedep/azure-ecs-rs and https://crates.io/crates/azure-ecs-rs instead of



This is a simple example of how to send an email using Azure Communication Service with REST API and SMTP.

It is developed according to this document.

[Azure Communication Service - Email - Rest API](https://learn.microsoft.com/en-us/rest/api/communication/email/send?tabs=HTTP)

How to create Azure Communication Service?

[Email Service](https://learn.microsoft.com/en-us/azure/communication-services/concepts/email/email-overview)

[Create Email Service](https://learn.microsoft.com/en-us/azure/communication-services/quickstarts/email/create-email-communication-resource)

For my example support Shared Key , Service Principal and Managed Identity.

How to run my example code , please setup environment variables follow this example below.
````

# For Shared Key
CONNECTION_STR="xxxxx"

# For SMTP
SMTP_USER="xxxx"
SMTP_PASSWORD="xxxx"
SMTP_SERVER="smtp.azurecomm.net"

# For Service Principle
CLIENT_ID="xx"
CLIENT_SECRET="xxx"
TENANT_ID="xxx"

ASC_URL="https://xxxxx.asiapacific.communication.azure.com"

# For Common
SENDER="xxx
REPLY_EMAIL="xxxx"
REPLY_EMAIL_DISPLAY="xxxx"

````

Get from Azure Portal

- CONNECTION_STR
![Alt text](https://github.com/preedep/rust_azure_email_communication/blob/develop/images/image2.png "Connection String")
- SENDER
![Alt text](https://github.com/preedep/rust_azure_email_communication/blob/develop/images/image1.png "Sender")

How to run my example code?
```
RUST_LOG=debug cargo run -- --help
```
```aiignore
Usage: azure_email_service [OPTIONS]

Options:
  -p, --protocol <PROTOCOL>        [default: rest] [possible values: rest, smtp]
  -a, --auth-method <AUTH_METHOD>  [default: shared-key] [possible values: managed-identity, service-principal, shared-key]
  -h, --help                       Print help
  -V, --version                    Print version
```
