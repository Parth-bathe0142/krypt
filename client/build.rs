fn main() {
    let profile = std::env::var("PROFILE").unwrap_or_default();

    let server_url = match profile.as_str() {
        "release" => "https://krypt-server-mdkb4odz.fermyon.app",
        _ => "http://localhost:3000",
    };

    println!("cargo:rustc-env=SERVER_URL={}", server_url);

    // Re-run this script if the build profile changes
    println!("cargo:rerun-if-env-changed=PROFILE");
}
