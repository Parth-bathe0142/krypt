
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use spin_sdk::sqlite3::Connection;
use anyhow::{Result};

use crate::util::get_salt;

fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];

    Argon2::default()
        .hash_password_into(password.as_bytes(), &salt, &mut key)
        .expect("argon2 failed");
    key
}

pub fn encrypt(plaintext: &str, password: &str, username: &str, conn: &Connection) -> Result<String> {
    let salt = get_salt(username, conn)?;
    
    let key = derive_key(password, salt.as_bytes());
    let cipher = Aes256Gcm::new(&key.into());

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .expect("encryption failed");

    // Store nonce prepended to ciphertext, base64 encoded
    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);
    Ok(B64.encode(combined))
}

pub fn decrypt(encoded: &str, password: &str, username: &str, conn: &Connection) -> anyhow::Result<String> {
    let salt = get_salt(username, conn)?;
    
    let key = derive_key(password, salt.as_bytes());
    let cipher = Aes256Gcm::new(&key.into());

    let combined = B64.decode(encoded)?;
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("Decryption failed — wrong password or corrupted data"))?;

    Ok(String::from_utf8(plaintext)?)
}
