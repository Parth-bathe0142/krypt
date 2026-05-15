fn main() {
    let profile = std::env::var("PROFILE").unwrap_or_default();

    let server_url = match profile.as_str() {
        "release" => "https://krypt.fermyon.app",
        _ => "http://localhost:3000",
    };

    println!("cargo:rustc-env=SERVER_URL={}", server_url);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PROFILE");
}
