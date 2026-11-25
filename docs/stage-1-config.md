# Stage 1: gateway config and `find_key`

**Goal:** the gateway loads its config (projects, users, API keys, providers) from a TOML file and looks up an incoming request's API key with `find_key`. Every later stage calls `find_key` first.

**The ownership rule for this stage:** `Config` owns everything. Everyone else borrows from it.

Full walkthrough: `~/selfstudy/rust/journey/01-ownership.md`, Part 8.

---

## Before you start: Pydantic's version (10 min)

In the build session, Claude walks you through `deploy/example.config.ts` and `gateway/src/types.ts`. What to notice:
- **Projects** have users and optional spending limits (daily/weekly/monthly).
- **API keys** belong to a project, optionally to a user, and list which **providers** they may use.
- **Providers** have a `providerId`, a `baseUrl`, and `credentials` read from an env var.
- In TS every object is a shared, mutable reference. In Rust you decide who owns each piece.

## Step 0: Create the repo ✅

Done: `cargo new`, `serde` (with `derive`), and `toml` are in `Cargo.toml`.

## Step 1: The types (`src/config.rs`)

Every field is owned (`String`, not `&str`). Be ready to say why.

| Struct | Fields |
|---|---|
| `Config` | `projects: Vec<Project>`, `users: Vec<User>`, `api_keys: Vec<ApiKey>`, `providers: Vec<ProviderConfig>` |
| `Project` | `id: u32`, `name: String`, `spending_limit_daily: Option<f64>` |
| `User` | `id: u32`, `project_id: u32`, `name: String`, `spending_limit_daily: Option<f64>` |
| `ApiKey` | `id: u32`, `key: String`, `project_id: u32`, `user_id: Option<u32>`, `providers: Vec<String>` |
| `ProviderConfig` | `name: String`, `provider_id: String`, `base_url: String`, `credentials_env: String` |

- `credentials_env` holds the **name** of the env var, never the secret. (`provider_id` becomes an enum in stage 2.)
- Derive `Debug, Deserialize`. Add `mod config;` to `main.rs`.
- **Test first:** build a `Config` by hand in a `#[test]` and assert `config.api_keys.len() == 1`.
- ⚠️ `#[derive(Debug)]` on `ApiKey` prints the raw key. Step 4 fixes that.

## Step 2: Load from TOML

- Write `config.example.toml` with `[[projects]]`, `[[users]]`, `[[api_keys]]`, `[[providers]]`. Fake keys only (`sk-gw-test-...`). Your real config goes in `gateway.toml`, which is gitignored.
- `fn from_toml(s: &str) -> Config`: `toml::from_str(s).expect("bad config")`. (`Result` comes in stage 2.)
- `fn load(path: &str) -> Config`: `std::fs::read_to_string`, then `from_toml`.
- **Test:** parse an inline `r#"..."#` string and check one field from each section.
- **Answer:** `from_toml` takes `&str`, but `Config` holds owned `String`s. Where did the bytes get copied?

## Step 3: `find_key`

```rust
impl Config {
    pub fn find_key(&self, key: &str) -> Option<&ApiKey> { todo!() }
}
```

- **Tests first:** a key that exists (`Some`, check its `id`) and one that doesn't (`None`).
- Write it with `for k in &self.api_keys { ... }`, then rewrite it as `self.api_keys.iter().find(...)`.
- **On purpose (E0507):** change the return type to `Option<ApiKey>`. Read the error. Change it back.
- **On purpose (E0502):** in a test, hold `let k = cfg.find_key(..)`, then `cfg.api_keys.push(..)`, then use `k`. Why is the compiler right?

## Step 4: Lookups that chain borrows

- `fn project_for(&self, key: &ApiKey) -> Option<&Project>`: follow `key.project_id`.
- `fn allows(&self, key: &ApiKey, provider: &str) -> bool`: does `key.providers` contain it? (`&String` vs `&str`: `.iter().any(|p| p == provider)` works.)
- **Redaction:** replace the derived `Debug` on `ApiKey` with a hand-written `impl std::fmt::Debug` that prints `key: "sk-gw-…"` (first 6 chars + ellipsis). Test that `format!("{:?}", k)` doesn't contain the full key.
- **On purpose (E0106):** write a free function `fn find_in(keys: &[ApiKey], key: &str) -> Option<&ApiKey>`. Elision fails with two input refs. Add `'a`. Should the result borrow from `keys` or `key`?

## Step 5: Wire `main`

```sh
cargo run -- check config.example.toml sk-gw-test-...
# key #3 → project "default", providers: [anthropic]
# or: unknown key
```

- Never print the key itself.

**Stretch:** a `HashMap<String, usize>` index (key → position in `api_keys`) so `find_key` is O(1). Why not store `HashMap<&str, &ApiKey>` inside `Config` itself? (Stage 5: self-referential structs.)

---

## Done when

- [ ] Config loads from TOML. `find_key`, `project_for`, and `allows` have passing tests.
- [ ] `cargo test` passes and `cargo clippy` has no warnings.
- [ ] You've triggered and read out loud **E0382, E0502, E0507, E0106**, and run `rustc --explain` on each.
- [ ] `Debug` output never shows a full key.
- [ ] In about 60 seconds, you can explain ownership and borrowing, `String` vs `&str`, and why `find_key` returns `Option<&ApiKey>`.

**Next:** `/rs-drill` for stage 1, then stage 2 (`enums-errors`): `provider_id` becomes an enum and `.expect` becomes `Result`.
