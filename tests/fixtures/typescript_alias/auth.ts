export function verifyToken(token: string): boolean {
  return token.length > 10;
}

export interface AuthConfig {
  secret: string;
}