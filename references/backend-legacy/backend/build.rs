fn main() {
    println!("cargo:rerun-if-env-changed=MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_JSON");
    println!("cargo:rerun-if-env-changed=MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_ID");
    println!("cargo:rerun-if-env-changed=MAKOSH_BUNDLED_GOOGLE_OAUTH_CLIENT_SECRET");
}
