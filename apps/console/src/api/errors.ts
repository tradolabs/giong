import { UnauthorizedError } from "./exception";

/**
 * Handles an API error by signing out on 401 or returning a user-facing message.
 */
export function getApiErrorMessage(
  error: unknown,
  onSignOut: () => void,
  fallback: string,
): string | null {
  if (error instanceof UnauthorizedError) {
    onSignOut();
    return null;
  }
  return error instanceof Error ? error.message : fallback;
}
