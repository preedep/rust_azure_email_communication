# Azure Email Communication

Prove of concept , Rust call Rest API of Azure Email Communication Service

It is developed according to this document.

[https://learn.microsoft.com/en-us/rest/api/communication/email/send?tabs=HTTP](Azure Communication Service - Email - Rest API)

````
export CONNECTION_STR="xxxxx-get-from-Azure-Portal" 
export RUST_LOG=info 
export SENDER="xxxx-get-from-Azure-Portal" 
export REPLY_EMAIL="xxx@abc.com" 
export REPLY_EMAIL_DISPLAY="xxxx@digital" 

cargo run

````

