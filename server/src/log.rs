#![allow(unused)]

fn now() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}


pub(crate) fn info(message: &str) {
    println!("[INFO] {}: {}", now(), message);
}

pub(crate) fn error(message: &str) {
    eprintln!("[ERROR] {}: {}", now(), message);
}

pub(crate) fn warn(message: &str) {
    println!("[WARN] {}: {}", now(), message);
}