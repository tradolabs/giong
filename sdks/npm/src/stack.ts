import {env} from "node:process";
import {type ReconcileOutput, ResourceConfig, ResourceInput} from "./types.ts";
import {readReconcileStream} from "./stream.ts";

export class Stack {
    private readonly resources: ResourceInput[] = [];
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

    add(name: string, config: ResourceConfig) {
        const input = new ResourceInput(name, config);
        if (this.resources.some(item => item.hashcode === input.hashcode)) {
            throw new Error(`Duplicate resource name: ${name}`);
        }
        this.resources.push(input);
    }

    async submit(): Promise<ReconcileOutput> {
        const response = await fetch(`${this.reconcilerUrl}/reconcile`, {
            method: "POST",
            headers: {"content-type": "application/json"},
            body: JSON.stringify({stack_id: this.stackId, resources: this.resources.map((item) => item.payload)}),
        });
        if (!response.ok) {
            throw new Error(`Reconciler returned ${response.status}: ${await response.text()}`);
        }
        const output = await readReconcileStream(response);
        console.log(JSON.stringify(output, null, 2));
        return output;
    }
}
