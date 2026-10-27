export function shortSha(sha: string): string {
  return sha.slice(0, 7);
}

export function formatRepo(url: string): string {
  return url.replace(/^https?:\/\/(www\.)?github\.com\//i, "").replace(/\.git$/i, "");
}
