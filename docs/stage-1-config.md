# Stage 1: gateway config and `find_key`

**Goal:** the gateway loads its config (projects, users, API keys, providers) from a TOML file and looks up an incoming request's API key with `find_key`. Every later stage calls `find_key` first.

**The ownership rule for this stage:** `Config` owns everything. Everyone else borrows from it.

**Design:** nested and keyed by id, like Pydantic's config. Users live inside their project, and API keys are keyed by the secret string itself.

Full walkthrough: `~/selfstudy/rust/journey/01-ownership.md`, Part 8 (it describes the flat `Vec` version; this file is the nested one you're building).

---

## Step 0: Create the repo ✅

## Step 1: The types (`src/config.rs`)

Every field is owned (`String`, not `&str`). Be ready to say why.

```
Config
├── projects:  HashMap<u32, Project>        // id → project
│   └── Project { name, users: HashMap<u32, User>, spending_limit_daily: Option<f64> }
│       └── User { name, spending_limit_daily: Option<f64> }
├── providers: HashMap<String, Provider>    // "anthropic" → provider
│   └── Provider { provider_id: ProviderId, base_url, inject_cost: bool, credentials_env }
└── api_keys:  HashMap<String, ApiKey>      // "sk-gw-…" → key
    └── ApiKey { id: u32, project: u32, user: Option<u32>, providers: Vec<String>,
                 spending_limit_total: Option<f64> }
```

- `credentials_env` holds the **name** of the env var (`"ANTHROPIC_API_KEY"`), never the secret.
- Use `f64` for every money field.
- Derive `Debug, Deserialize` on all of them, including `Config`. Add `mod config;` to `main.rs`.
- **Test first:** build a `Config` by hand (struct literals, `HashMap::from([...])`) with one of each, and assert `config.api_keys.len() == 1`.

## Step 2: Load from TOML

`config.example.toml` (fake keys only; your real one is `gateway.toml`, which is gitignored):

```toml
[projects.1]
name = "default"
spending_limit_daily = 10.0

[projects.1.users.2]
name = "Samuel"

[providers.anthropic]
provider_id = "anthropic"
base_url = "https://api.anthropic.com"
inject_cost = true
credentials_env = "ANTHROPIC_API_KEY"

[api_keys."sk-gw-test-abc123"]
id = 3
project = 1
user = 2
providers = ["anthropic"]
```

- `fn from_toml(s: &str) -> Config`: `toml::from_str(s).expect("bad config")`.
- `fn load(path: &str) -> Config`: `std::fs::read_to_string`, then `from_toml`.
- **Test:** parse an inline `r#"..."#` string and check one field from each section, including the nested user.
- **Answer:** `from_toml` takes `&str`, but `Config` holds owned `String`s. Where did the bytes get copied?

## Step 3: `find_key`

```rust
impl Config {
    pub fn find_key(&self, key: &str) -> Option<&ApiKey> { todo!() }
}
```

- **Tests first:** a key that exists (`Some`, check its `id`) and one that doesn't (`None`).
- With a `HashMap` this is one call. Look at what `HashMap::get` returns, and notice you can pass a `&str` to a map whose keys are `String`.
- **On purpose (E0507):** change the return type to `Option<ApiKey>` and try to make it compile by dereferencing (`.map(|k| *k)`). Read the error. Change it back.
- **On purpose (E0502):** in a test, hold `let k = cfg.find_key(..)`, then `cfg.api_keys.insert(..)`, then use `k`. Why is the compiler right? (Hint: what can `insert` do to the map's memory?)

## Step 4: Lookups that chain borrows

- `fn project_for(&self, key: &ApiKey) -> Option<&Project>`: follow `key.project`.
- `fn user_for(&self, key: &ApiKey) -> Option<&User>`: two hops, `key.user` (an `Option`) then the project's `users` map. Every reference you return borrows from `self`.
- `fn allows(&self, key: &ApiKey, provider: &str) -> bool`: does `key.providers` contain it? (`&String` vs `&str`: `.iter().any(|p| p == provider)` works.)
- **Redaction:** the secret is now a map key, so a derived `Debug` on **`Config`** prints every secret. Replace it with a hand-written `impl std::fmt::Debug for Config` that shows each key as `"sk-gw-…"` (first 6 chars + ellipsis). Test that `format!("{:?}", cfg)` doesn't contain the full key.
- **On purpose (E0106):** write a free function `fn find_in(keys: &HashMap<String, ApiKey>, key: &str) -> Option<&ApiKey>`. Elision fails with two input refs. Add `'a`. Should the result borrow from `keys` or `key`?

## Step 5: Wire `main`

```sh
cargo run -- check config.example.toml sk-gw-test-abc123
# key #3 → project "default", user "Samuel", providers: [anthropic]
# or: unknown key
```

- Never print the key itself.

**Stretch:** `find_key` is already O(1). Instead, try a method that **mutates** while it reads: `fn revoke(&mut self, key: &str) -> Option<ApiKey>` that removes and returns the key. Why can this one return an owned `ApiKey` when `find_key` couldn't?

---

## Done when

- [ ] Config loads from TOML. `find_key`, `project_for`, `user_for`, and `allows` have passing tests.
- [ ] `cargo test` passes and `cargo clippy` has no warnings.
- [ ] You've triggered and read out loud **E0382, E0502, E0507, E0106**, and run `rustc --explain` on each.
- [ ] `Debug` output never shows a full key.
- [ ] In about 60 seconds, you can explain ownership and borrowing, `String` vs `&str`, and why `find_key` returns `Option<&ApiKey>`.

**Next:** `/rs-drill` for stage 1, then stage 2 (`enums-errors`): `.expect` becomes `Result`.
