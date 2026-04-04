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
