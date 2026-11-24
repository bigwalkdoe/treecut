use crate::auth::verify_token as check_tok;

pub fn handle_request(token: &str) -> bool {
    check_tok(token)
}