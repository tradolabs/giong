export function toSnakeCase(value: unknown): unknown {
    if (Array.isArray(value)) {
        return value.map(toSnakeCase);
    }
    if (value && typeof value === "object") {
        return Object.fromEntries(
            Object.entries(value).map(([k, v]) => [
                k.replace(/[A-Z]/g, letter => `_${letter.toLowerCase()}`),
                toSnakeCase(v),
            ]),
        );
    }
    return value;
}
