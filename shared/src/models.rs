use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use spin_sdk::{
    http::Request,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}
impl Credentials {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}

#[derive(Serialize,Deserialize, Debug)]
pub struct Key {
    pub name: String,
    pub value: Option<String>,
}
impl Key {
    pub fn new(name: String, value: Option<String>) -> Self {
        Self { name, value }
    }
}

#[derive(Serialize,Deserialize, Debug)]
pub struct KeyPayload {
    pub creds: Credentials,
    pub key: Key,
}
impl KeyPayload {
    pub fn new(creds: Credentials, key: Key) -> Self {
        Self { creds, key }
    }
}

#[derive(Serialize,Deserialize, Debug)]
pub struct ChangePasswordPayload {
    pub creds: Credentials,
    pub new_password: String,
}
impl ChangePasswordPayload {
    pub fn new(creds: Credentials, new_password: String) -> Self {
        Self { creds, new_password }
    }
}

#[derive(Serialize,Deserialize)]
pub struct ChangeKeyPayload {
    pub creds: Credentials,
    pub name: String,
    pub new_value: String
}
impl ChangeKeyPayload {
    pub fn new(creds: Credentials, name: String, new_value: String) -> Self {
        Self { creds, name, new_value }
    }
}

pub trait JsonPayload: for<'a> Deserialize<'a> {
    fn from_request(req: Request) -> Result<Self> {
        let str = String::from_utf8(req.into_body())
            .map_err(|_| anyhow!("request missing body".to_string()))?;

        serde_json::from_str::<Self>(&str)
            .map_err(|_| anyhow!("Could not parse request body".to_string()))
    }

    #[allow(unused)]
    fn from_request_parts(req: &Request) -> Result<Self> {
        let str = String::from_utf8(req.body().to_owned())
            .map_err(|_| anyhow!("request missing body".to_string()))?;

        serde_json::from_str::<Self>(&str)
            .map_err(|_| anyhow!("Could not parse request body".to_string()))
    }
}
impl<T: for<'a> Deserialize<'a>> JsonPayload for T {}

pub trait ToJson: Serialize {
    fn to_json_string(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|_| anyhow!("Could not serialize to JSON".to_string()))
    }
}
impl<T: Serialize> ToJson for T {}