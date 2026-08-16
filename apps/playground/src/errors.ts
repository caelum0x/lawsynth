export type PlaygroundErrorCode = "invalid-world" | "invalid-dataset" | "wasm-unavailable" | "simulation-failed" | "storage-failed" | "share-failed" | "cancelled" | "limit-exceeded";
export class PlaygroundError extends Error {
    constructor(readonly code: PlaygroundErrorCode, message: string, override readonly cause?: unknown) {
        super(message, cause === undefined ? undefined : { cause });
        this.name = "PlaygroundError";
    }
}
export function normalizePlaygroundError(error: unknown, fallback: PlaygroundErrorCode = "simulation-failed"): PlaygroundError {
    if (error instanceof PlaygroundError)
        return error;
    if (error instanceof DOMException && error.name === "AbortError")
        return new PlaygroundError("cancelled", error.message || "Operation cancelled", error);
    if (error instanceof Error)
        return new PlaygroundError(fallback, error.message, error);
    return new PlaygroundError(fallback, String(error));
}
export function userErrorMessage(error: PlaygroundError): string {
    switch (error.code) {
        case "invalid-world": return `The world definition is not valid: ${error.message}`;
        case "invalid-dataset": return `The dataset cannot be used: ${error.message}`;
        case "wasm-unavailable": return "The local simulation runtime could not be loaded. Check browser WebAssembly support and the application deployment.";
        case "simulation-failed": return `Simulation stopped: ${error.message}`;
        case "storage-failed": return "The browser could not save this playground. Export a copy before leaving the page.";
        case "share-failed": return "A share link could not be created. The current work remains local.";
        case "cancelled": return "The operation was cancelled.";
        case "limit-exceeded": return `The requested operation exceeds a browser safety limit: ${error.message}`;
    }
}
