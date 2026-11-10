export interface Runner {
  id: string;
  name: string;
  version: string;
  description: string;
  defaults: Record<string, unknown>;
  configuration_schema: Record<string, unknown>;
}
