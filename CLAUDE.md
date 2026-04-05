# SEO Insikt — Project Notes

## Stack
This is a **Tauri** desktop app (Rust backend + Next.js frontend).

## Running the app
```
npm run tauri dev
```
Do NOT use `cargo run` or `npm run dev` alone — always use the Tauri command above.

## Bindings
`src/bindings.ts` is **auto-generated** by tauri-specta every time `npm run tauri dev` runs. **Never edit it manually** — any changes will be overwritten.

## Database / SQLx
CI uses `SQLX_OFFLINE=true` — no database server is needed to build or test.

- If you add or change a `sqlx::query!()` macro, regenerate the cache before pushing:
  ```
  cd src-tauri
  DATABASE_URL=sqlite:./dev.db cargo sqlx prepare
  ```
  Then commit the updated files in `src-tauri/.sqlx/`.

- The `extension_repository.rs` uses runtime `sqlx::query()` calls (not macros), so changes there don't require regenerating the cache.
- Integration tests use `sqlite::memory:` — no external DB needed.
