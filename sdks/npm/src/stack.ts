import {env} from "node:process";
import {type ReconcileOutput, type Resource, ResourceConfig} from "./types.ts";

export function Stack() {
    const resources: Resource[] = [];
    const reconcilerUrl = env.GIONG_RECONCILER_URL;

    if (!reconcilerUrl) {
        throw new Error("GIONG_RECONCILER_URL is required");
    }

    return {
        add(name: string, config: ResourceConfig) {
            if (resources.some(resource => resource.name === name)) {
                throw new Error(`Duplicate resource name: ${name}`);
            }
            resources.push({name, type: config.type, config: config});
        },

        async submit(): Promise<ReconcileOutput> {
            const response = await fetch(`${reconcilerUrl}/reconcile`, {
                method: "POST",
                headers: {"content-type": "application/json"},
                body: JSON.stringify({resources}),
            });
            if (!response.ok) {
                throw new Error(`Reconciler returned ${response.status}: ${await response.text()}`);
            }
            return response.json() as Promise<ReconcileOutput>;
        },
    };
}
