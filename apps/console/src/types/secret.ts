export interface Secret {
  key: string;
  source: string | null;
}
export interface SecretSet {
  id: string;
  name: string;
  type: "shared" | "local";
}
