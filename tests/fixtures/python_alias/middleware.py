from auth import verify_token as check_tok

def handle_request(token: str) -> bool:
    return check_tok(token)