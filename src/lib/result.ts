import { logger } from "./logger";

export class Result<T, E = string> {
	private constructor(
		private readonly _tag: 'Ok' | 'Err',
		private readonly _value?: T,
		private readonly _error?: E
	) { }

	/* constructors */
	static Ok<T>(value: T): Result<T, never> { return new Result<T, never>('Ok', value); }
	static Err<E>(error: E): Result<never, E> { return new Result<never, E>('Err', undefined, error); }

	/* predicates */
	isOk(): this is Ok<T> { return this._tag === 'Ok'; }
	isErr(): this is Err<E> { return this._tag === 'Err'; }

	/* extractors */
	unwrap(): T {
		if (this.isOk()) return this._value as T;
		throw new Error(`unwrap on Err: ${this._error}`);
	}
	expect(msg: string): T {
		if (this.isOk()) return this._value as T;
		throw new Error(`${msg}: ${this._error}`);
	}
	unwrapOr(fb: T): T { return this.isOk() ? (this._value as T) : fb; }

	/* combinators */
	map<U>(fn: (v: T) => U): Result<U, E> {
		return this.isOk() ? Result.Ok(fn(this._value as T)) : (this as any);
	}
	mapErr<F>(fn: (e: E) => F): Result<T, F> {
		return this.isErr() ? Result.Err(fn(this._error as E)) : (this as any);
	}
	andThen<U>(fn: (v: T) => Result<U, E>): Result<U, E> {
		return this.isOk() ? fn(this._value as T) : (this as any);
	}
	match<U>(onOk: (v: T) => U, onErr: (e: E) => U): U {
		return this.isOk() ? onOk(this._value as T) : onErr(this._error as E);
	}
	async matchAsync<U>(
		onOk: (val: T) => U | Promise<U>,
		onErr: (err: E) => U | Promise<U>
	): Promise<U> {
		logger.info(this._value)
		return this.isOk()
			? await onOk(this._value as T)
			: await onErr(this._error as E);
	}
}

export type Ok<T> = Result<T, never>;
export type Err<E> = Result<never, E>;

export async function fromPromise<T>(p: Promise<T>): Promise<Result<T, string>> {
	return p
		.then((v) => {
			logger.info(v);
			return Result.Ok(v)
		})
		.catch((e) => {
			logger.warn(e)
			return Result.Err((e as Error)?.message ?? String(e))
		});
}
