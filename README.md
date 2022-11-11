# Azure Email Communication

Prove of concept , Rust call Rest API of Azure Email Communication Service

It is developed according to this document.

[Azure Communication Service - Email - Rest API](https://learn.microsoft.com/en-us/rest/api/communication/email/send?tabs=HTTP)

How to create Azure Communication Service?

[Email Service](https://learn.microsoft.com/en-us/azure/communication-services/concepts/email/email-overview)

[Create Email Service](https://learn.microsoft.com/en-us/azure/communication-services/quickstarts/email/create-email-communication-resource)


How to run my example code , please setup environment variables follow this example below.
````
export CONNECTION_STR="xxxxx-get-from-Azure-Portal" 
export RUST_LOG=info 
export SENDER="xxxx-get-from-Azure-Portal" 
export REPLY_EMAIL="xxx@abc.com" 
export REPLY_EMAIL_DISPLAY="xxxx@digital" 

cargo run

````

Get from Azure Portal

- CONNECTION_STR
![Alt text](https://github.com/preedep/rust_azure_email_communication/blob/develop/images/image2.png "Connection String")
- SENDER
![Alt text](https://github.com/preedep/rust_azure_email_communication/blob/develop/images/image1.png "Sender")


