export abstract class ResourceConfig {
    type: string

    constructor(type: string) {
        this.type = type;
    }
}

export type Resource = {
    name: string;
    type: string;
    config: ResourceConfig;
}

export type ResourceDescriptor = Pick<Resource, 'name' | 'type'>;

export interface ReconcileOutput {
    created: ResourceDescriptor[];
    unchanged: ResourceDescriptor[];
}
