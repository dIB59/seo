// src/lib/logger.ts
const dev = process.env.NODE_ENV === "development";

const stamp = () => `[${new Date().toLocaleTimeString()}]`;

type LogFn = typeof console.log;
const noop: LogFn = () => { };

export const logger = {
	log: dev ? (...a: unknown[]) => console.log("[LOG]", stamp(), ...a) : noop,
	info: dev ? (...a: unknown[]) => console.info("[INFO]", stamp(), ...a) : noop,
	warn: dev ? (...a: unknown[]) => console.warn("[WARN]", stamp(), ...a) : noop,
	error: (...a: unknown[]) => console.error("[ERROR]", stamp(), ...a),
	group: dev ? (label?: string) => console.group(`[GRP] ${stamp()} ${label ?? ""}`) : noop,
	groupEnd: dev ? () => console.groupEnd() : noop,
} as const;
