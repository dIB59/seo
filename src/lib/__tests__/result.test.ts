import { describe, it, expect } from "vitest";
import { Result, fromPromise } from "@/src/lib/result";

describe("Result", () => {
  it("Ok carries a value and reports isOk", () => {
    const r = Result.Ok(42);
    expect(r.isOk()).toBe(true);
    expect(r.isErr()).toBe(false);
    expect(r.unwrap()).toBe(42);
    expect(r.unwrapOr(0)).toBe(42);
  });

  it("Err carries an error and reports isErr", () => {
    const r = Result.Err("boom");
    expect(r.isErr()).toBe(true);
    expect(r.isOk()).toBe(false);
    expect(r.unwrapErr()).toBe("boom");
    expect(r.unwrapOr(7 as never)).toBe(7);
  });

  it("unwrap on Err throws", () => {
    expect(() => Result.Err("nope").unwrap()).toThrow(/unwrap on Err/);
  });

  it("map transforms Ok and passes through Err", () => {
    expect(Result.Ok(2).map((n) => n * 3).unwrap()).toBe(6);
    const err = Result.Err<string>("e").map((n: number) => n + 1);
    expect(err.isErr()).toBe(true);
  });

  it("andThen chains Results", () => {
    const r = Result.Ok(2).andThen((n) => Result.Ok(n + 1));
    expect(r.unwrap()).toBe(3);
  });

  it("match dispatches on tag", () => {
    expect(Result.Ok(1).match((v) => `ok:${v}`, (e) => `err:${e}`)).toBe("ok:1");
    expect(Result.Err("x").match((v) => `ok:${v}`, (e) => `err:${e}`)).toBe("err:x");
  });

  it("fromPromise wraps resolved promises into Ok", async () => {
    const r = await fromPromise(Promise.resolve("hi"));
    expect(r.isOk()).toBe(true);
    expect(r.unwrap()).toBe("hi");
  });

  it("fromPromise wraps rejected promises into Err", async () => {
    const r = await fromPromise(Promise.reject(new Error("nope")));
    expect(r.isErr()).toBe(true);
    expect(r.unwrapErr()).toBe("nope");
  });
});
