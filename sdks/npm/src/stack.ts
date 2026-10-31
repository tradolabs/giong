import {env} from "node:process";
import type {ReconcileInput, ReconcileOutput, Resource, Provider} from "./types.ts";
import {readReconcileStream} from "./stream.ts";

export class Stack {
    private readonly providers: Map<string, Provider> = new Map();
    private readonly resources: Resource[] = [];
    private readonly reconcilerUrl: string;
    private readonly stackId: string;

    constructor() {
        const reconcilerUrl = env.GIONG_RECONCILER_URL;
        const stackId = env.GIONG_STACK_ID;

        if (!reconcilerUrl) {
            throw new Error("GIONG_RECONCILER_URL is required");
        }

        if (!stackId) {
            throw new Error("GIONG_STACK_ID is required");
        }

        this.reconcilerUrl = reconcilerUrl;
        this.stackId = stackId;
    }

    add(resource: Resource) {
        this.providers.set(resource.provider.urn, resource.provider);
        this.resources.push(resource);
    }

    async submit(): Promise<ReconcileOutput> {
        const input: ReconcileInput = {
            stackId: this.stackId,
            providers: Array.from(this.providers.values()).map((provider) => ({
                urn: provider.urn,
                inputs: provider.inputs,
            })),
            resources: this.resources.map(resource => ({
                name: resource.name,
                package: resource.package,
                provider: resource.provider.urn,
                inputs: resource.inputs,
            })),
        };
        const response = await fetch(`${this.reconcilerUrl}/reconcile`, {
            method: "POST",
            headers: {"content-type": "application/json"},
            body: JSON.stringify(input),
        });
        if (!response.ok) {
            throw new Error(`Reconciler returned ${response.status}: ${await response.text()}`);
        }
        const output = await readReconcileStream(response);
        console.log(JSON.stringify(output, null, 2));
        return output;
    }
}
