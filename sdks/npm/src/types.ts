export abstract class ResourceConfig {
    type: string

    constructor(type: string) {
        this.type = type;
    }
}

export type Resource = {
    name: string;
    type: string;
    config: object;
}

export interface ReconcileOutput {
    created: Resource[];
    unchanged: Resource[];
}
