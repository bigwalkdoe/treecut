use crate::auth::{verify_token as check_tok, refresh_token};

pub fn handle_request(token: &str) -> bool {
    check_tok(token)
}

pub fn refresh_session(token: &str) -> String {
    refresh_token(token)
}