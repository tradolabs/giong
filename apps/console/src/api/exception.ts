export class UnauthorizedError extends Error {
  constructor() {
    super("Session expired or unauthorized.");
  }
}
