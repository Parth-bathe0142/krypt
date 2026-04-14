use anyhow::{anyhow, Result};
use spin_sdk::key_value::Store;

use crate::util::now;

const MAX_ATTEMPTS: u32 = 5;
const WINDOW: u64 = 60;

struct Attempts {
    count: u32,
    expires_at: u64,
}

impl Attempts {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let s = std::str::from_utf8(bytes).ok()?;
        let mut parts = s.split(':');

        Some(Self {
            count: parts.next()?.parse().ok()?,
            expires_at: parts.next()?.parse().ok()?,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        format!("{}:{}", self.count, self.expires_at).into_bytes()
    }
}

pub fn check_rate_limit(username: &str) -> Result<()> {
    let store = Store::open_default()?;
    let key = format!("rate:{}", username);
    let now = now();

    let attempts = store
        .get(&key)?
        .and_then(|b| Attempts::from_bytes(&b))
        .filter(|a| a.expires_at > now)
        .unwrap_or(Attempts {
            count: 0,
            expires_at: now + WINDOW,
        });

    if attempts.count >= MAX_ATTEMPTS {
        return Err(anyhow!("Too many attempts, try again later"));
    }

    store.set(
        &key,
        &Attempts {
            count: attempts.count + 1,
            ..attempts
        }
        .to_bytes(),
    )?;
    Ok(())
}

pub fn clear_rate_limit(username: &str) -> Result<()> {
    let store = Store::open_default()?;
    store.delete(&format!("rate:{}", username))?;
    Ok(())
}
