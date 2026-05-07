use aes_gcm::{
    aead::{rand_core::RngCore, Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use spin_sdk::sqlite::Connection;

use crate::{log, util::text};

fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];

    Argon2::default()
        .hash_password_into(password.as_bytes(), &salt, &mut key)
        .map_err(|err| {
            log::error(&format!("Failed to derive key: {}", err));
            anyhow!("Failed to derive key: {}", err)
        })?;

    Ok(key)
}

 fn get_salt(username: &str, conn: &Connection) -> Result<Vec<u8>> {
    Ok(conn
        .execute(
            "select salt from Accounts where username = ?",
            &[text(username)],
        )?
        .rows
        .first()
        .unwrap()
        .get::<&[u8]>(0)
        .map(ToOwned::to_owned)
        .unwrap())
}

pub fn encrypt(
    plaintext: &str,
    password: &str,
    username: &str,
    conn: &Connection,
) -> Result<String> {
    let salt = get_salt(username, conn)?;

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key.into());

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).map_err(|err| {
        log::error(&format!("Encryption failed: {}", err));
        anyhow!("Encryption failed: {}", err)
    })?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);
    Ok(B64.encode(combined))
}

pub fn decrypt(encoded: &str, password: &str, username: &str, conn: &Connection) -> Result<String> {
    let salt = get_salt(username, conn)?;

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new(&key.into());

    let combined = B64.decode(encoded)?;
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|err| {
        log::error(&format!("Decryption failed: {}", err));
        anyhow!("Decryption failed: {}", err)
    })?;

    Ok(String::from_utf8(plaintext)?)
}
