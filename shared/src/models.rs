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

#[derive(Serialize,Deserialize, Debug)]
pub struct Key {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Serialize,Deserialize, Debug)]
pub struct KeyPayload {
    pub creds: Credentials,
    pub key: Key,
}

#[derive(Serialize,Deserialize, Debug)]
pub struct ChangePasswordPayload {
    pub creds: Credentials,
    pub new_password: String,
}

#[derive(Serialize,Deserialize)]
pub struct ChangeKeyPayload {
    pub creds: Credentials,
    pub name: String,
    pub new_value: String
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
