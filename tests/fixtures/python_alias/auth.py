def verify_token(token: str) -> bool:
    return len(token) > 10

class AuthConfig:
    def __init__(self, secret: str):
        self.secret = secret