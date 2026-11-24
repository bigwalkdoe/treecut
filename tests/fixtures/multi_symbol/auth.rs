pub fn verify_token(token: &str) -> bool {
    token.len() > 10
}

pub fn refresh_token(token: &str) -> String {
    format!("refreshed_{}", token)
}

pub struct AuthConfig {
    pub secret: String,
}