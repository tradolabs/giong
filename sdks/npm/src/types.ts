export abstract class Component {
    readonly name: string;

    constructor(name: string) {
        this.name = name;
    }

    get urn(): string {
        return `${this.package}::${this.name}`;
    }

    abstract get package(): string;

    abstract get inputs(): Record<string, unknown>;
}

export abstract class Provider extends Component {
}

export abstract class Resource<T extends Provider = Provider> extends Component {
    readonly provider: T;

    constructor(name: string, provider: T) {
        super(name);
        this.provider = provider;
    }
}

export type ReconcileInput = {
    stackId: string;
    providers: {
        urn: string;
        inputs: Record<string, unknown>;
    }[];
    resources: {
        name: string;
        package: string;
        provider: string;
        inputs: Record<string, unknown>;
    }[];
};

export type ReconcileOutput = Record<"created" | "updated" | "unchanged", {
    name: string;
    package: string;
    provider?: string;
    inputs: unknown;
}[]>;
