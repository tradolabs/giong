import {toSnakeCase} from "./utils.ts";

export abstract class ResourceConfig {
    abstract get type(): string;
}

export class ResourceInput {
    name: string;
    config: ResourceConfig;

    constructor(name: string, config: ResourceConfig) {
        this.name = name;
        this.config = config;
    }

    get hashcode(): string {
        return `${this.config.type}/${this.name}`;
    }

    get payload(): Record<string, any> {
        return {
            name: this.name,
            type: this.config.type,
            config: toSnakeCase(this.config)
        }
    }
}

export type Resource = {
    name: string;
    type: string;
    config: object;
}

export interface ReconcileOutput {
    created: Resource[];
    updated: Resource[];
    unchanged: Resource[];
}
