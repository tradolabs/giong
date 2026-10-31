import {createInterface} from "node:readline";
import {Readable} from "node:stream";
import type {ReadableStream} from "node:stream/web";
import type {ReconcileOutput} from "./types.ts";

type ReconcileEvent =
    | {type: "log"; data: string}
    | {type: "error"; data: string}
    | {type: "complete"; data: ReconcileOutput};

export async function readReconcileStream(response: Response): Promise<ReconcileOutput> {
    if (!response.body) {
        throw new Error("Reconciler returned an empty stream");
    }
    const input = Readable.fromWeb(response.body as ReadableStream<Uint8Array>);
    const lines = createInterface({input, crlfDelay: Infinity});
    try {
        for await (const line of lines) {
            if (!line.trim()) continue;
            const event: ReconcileEvent = JSON.parse(line);
            switch (event.type) {
                case "log":
                    console.log(event.data);
                    break;
                case "error":
                    throw new Error(event.data);
                case "complete":
                    return event.data;
                default:
                    throw new Error("Unknown reconcile event");
            }
        }
        throw new Error("Reconcile stream ended without a result");
    } finally {
        lines.close();
        input.destroy();
    }
}
