# llm-gateway

A local LLM gateway in Rust. Claude Code and Pydantic AI scripts send their traffic through it. It mirrors the [Pydantic AI Gateway](https://github.com/pydantic/pydantic-ai-gateway) feature by feature.

The curriculum, progress and notes live in `~/selfstudy/rust/` (`curriculum.md`, `progress.md`). This file lists only the references.

## The design source

**[pydantic/pydantic-ai-gateway](https://github.com/pydantic/pydantic-ai-gateway)**: TypeScript on Cloudflare Workers, archived 2026-03.
- It's licensed **AGPL**, so read it for the design and never copy code.
- You don't need to read TypeScript. `/rs-build` translates each file into a Pydantic-style sketch with example data.

| Stage | Pydantic file(s) | What it shows |
|---|---|---|
| 1 `ownership` | `deploy/example.config.ts`, `gateway/src/types.ts` | Config shape: projects, users, API keys, providers |
| 2 `enums-errors` | `gateway/src/providers/base.ts`, `providers/anthropic.ts`, `gateway/src/handler.ts` | Provider auth headers, Anthropic response/usage, error paths |
| 3 `iterators-closures` | `gateway/src/refreshGenaiPrices.ts` | How prices are loaded |
| 4 `traits-generics` | `gateway/src/providers/base.ts`, `gateway/src/cache/adapter.d.ts` | Provider base class, cache interface |
| 6 `smart-pointers` | `gateway/src/cache/index.ts` | Response caching (theirs: Redis/KV; ours: in-memory LRU) |
| 7 `threads-sync` | `gateway/src/rateLimiter.ts` | Rate limiting |
| 8 `async` | `gateway/src/gateway.ts`, `gateway/src/handler.ts`, `proxy-vcr/` | Request flow; record/replay tests without API spend |
| 11 `production` | `gateway/src/otel/` (`attributes.ts`, `genai.d.ts`) | OpenTelemetry GenAI span attributes |

Stages 5, 9, 10, 12 and 13 have no matching Pydantic file.

Fetch any file without cloning:

```sh
gh api repos/pydantic/pydantic-ai-gateway/contents/<path> --jq .content | base64 -d
```

## Other projects

- **[pydantic/genai-prices](https://github.com/pydantic/genai-prices)** (stage 3): per-model token prices. The data is under `prices/`.
- **[pydantic/pydantic-core](https://github.com/pydantic/pydantic-core)** (stage 12): the Rust core with a Python API that stage 12 copies. The repo is archived, but the layout still applies. Rust is in `src/`, the Python package is in `python/`, and it builds with maturin.
- **[Anthropic Messages API: streaming](https://docs.anthropic.com/en/api/messages-streaming)** (stages 2, 9): SSE event types and where token usage appears.

## Rust reading by stage

| Stage | Reading |
|---|---|
| 0 `basics` | Rust Book ch. 1–3; Rustlings `intro` → `vecs` |
| 1 `ownership` | Rust Book ch. 4, 5, 8 |
| 2 `enums-errors` | Rust Book ch. 6, 9; [serde docs](https://serde.rs) |
| 3 `iterators-closures` | Rust Book ch. 13 |
| 4 `traits-generics` | Rust Book ch. 10, 17, 19.2 |
| 5 `lifetimes` | Rust Book ch. 10.3; Crust of Rust: Lifetime Annotations |
| 6 `smart-pointers` | Rust Book ch. 15; *Learning Rust With Entirely Too Many Linked Lists* |
| 7 `threads-sync` | Rust Book ch. 16; Mara Bos, *Rust Atomics and Locks* ch. 1–3 |
| 8 `async` | [tokio tutorial](https://tokio.rs/tokio/tutorial); Crust of Rust: async/await |
| 9 `streaming-pin` | *Asynchronous Programming in Rust* (Pin chapter); fasterthanli.me "Pin and suffering" |
| 10 `tower-middleware` | "Inventing the Service trait" (tokio blog); [tower docs](https://docs.rs/tower) |
| 13 `depth` | *Rust for Rustaceans* |

Links: [Rust Book](https://doc.rust-lang.org/book/) · [Rustlings](https://github.com/rust-lang/rustlings) · [Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/) · [Rust Atomics and Locks](https://marabos.nl/atomics/) · `rustc --explain <code>` for any error code.

## Hooking up traffic

- **Claude Code:** `ANTHROPIC_BASE_URL=http://127.0.0.1:<port>`. Unset it to go direct again. Check in stage 7 or 8 whether this works with your login type.
- **Pydantic AI:** point the Anthropic provider's base URL at the gateway.

The gateway listens on 127.0.0.1 only, and it never logs API keys or auth headers.
