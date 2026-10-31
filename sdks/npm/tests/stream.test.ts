import assert from "node:assert/strict";
import {test} from "node:test";
import {readReconcileStream} from "../src/stream.ts";

const output = {created: [], updated: [], unchanged: []};
const encoder = new TextEncoder();

test("prints logs before completion and handles fragmented UTF-8 events", async (t) => {
    const log = t.mock.method(console, "log", () => {});
    let controller!: ReadableStreamDefaultController<Uint8Array>;
    const response = new Response(new ReadableStream<Uint8Array>({
        start(value) { controller = value; },
    }));
    const result = readReconcileStream(response);
    const bytes = encoder.encode(JSON.stringify({type: "log", data: "Đang xử lý"}) + "\n");
    for (const byte of bytes) controller.enqueue(new Uint8Array([byte]));
    await new Promise(resolve => setImmediate(resolve));
    assert.equal(log.mock.callCount(), 1);
    assert.deepEqual(log.mock.calls[0].arguments, ["Đang xử lý"]);
    controller.enqueue(encoder.encode(JSON.stringify({type: "complete", data: output})));
    controller.close();
    assert.deepEqual(await result, output);
});

test("rejects a streamed provider error", async () => {
    const response = new Response(JSON.stringify({type: "error", data: "apply failed"}) + "\n");
    await assert.rejects(readReconcileStream(response), /apply failed/);
});

test("rejects a stream that ends without a result", async () => {
    await assert.rejects(readReconcileStream(new Response("")), /ended without a result/);
});

test("rejects malformed events", async () => {
    await assert.rejects(readReconcileStream(new Response("broken\n")), SyntaxError);
});

test("propagates a connection failure", async () => {
    const response = new Response(new ReadableStream({
        pull(controller) { controller.error(new Error("connection lost")); },
    }));
    await assert.rejects(readReconcileStream(response), /connection lost/);
});

test("cancels the response body after the final result", async () => {
    let cancelled = false;
    const response = new Response(new ReadableStream({
        start(controller) {
            controller.enqueue(encoder.encode(JSON.stringify({type: "complete", data: output}) + "\n"));
        },
        cancel() { cancelled = true; },
    }));
    assert.deepEqual(await readReconcileStream(response), output);
    assert.equal(cancelled, true);
});
