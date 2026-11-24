import { verifyToken as checkTok } from './auth';

export function handleRequest(token: string): boolean {
  return checkTok(token);
}