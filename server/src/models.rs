use bcrypt::verify;
use serde::{Deserialize, Serialize};
use spin_sdk::sqlite3::{Connection, Value};

#[derive(Serialize, Deserialize, Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}
impl Credentials {
    pub fn verify(&self, conn: &Connection) -> anyhow::Result<bool> {
        let rows = conn.execute(
            "select pass_hash from Accounts where username = ?",
            &[Value::Text(self.username.clone())],
        )?;

        let Some(row) = rows.rows.first() else {
            return Ok(false);
        };

        let pass_hash = row.get::<&str>(0).unwrap();
        verify(&self.password, pass_hash).map_err(Into::into)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Key {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyPayload {
    pub creds: Credentials,
    pub key: Key,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangePasswordPayload {
    pub creds: Credentials,
    pub new_password: String,
}