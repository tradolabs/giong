import {env} from "node:process";
import {type ReconcileOutput, ResourceConfig, ResourceInput} from "./types.ts";

export class Stack {
    private readonly resources: ResourceInput[] = [];
    private readonly reconcilerUrl: string;

    constructor() {
        const reconcilerUrl = env.GIONG_RECONCILER_URL;
        if (!reconcilerUrl) {
            throw new Error("GIONG_RECONCILER_URL is required");
        }
        this.reconcilerUrl = reconcilerUrl;
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
            body: JSON.stringify(this.resources.map((item) => item.payload)),
        });
        if (!response.ok) {
            throw new Error(`Reconciler returned ${response.status}: ${await response.text()}`);
        }
        const output: ReconcileOutput = await response.json();
        console.log(JSON.stringify(output, null, 2));
        return output;
    }
}
