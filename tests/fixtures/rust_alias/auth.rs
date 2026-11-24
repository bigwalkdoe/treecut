pub fn verify_token(token: &str) -> bool {
    token.len() > 10
}

pub struct AuthConfig {
    pub secret: String,
}