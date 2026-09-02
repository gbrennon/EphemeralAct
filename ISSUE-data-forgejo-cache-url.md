# Issue: `uses: https://data.forgejo.org/actions/cache@v4` fails and `${{ secrets.* }}` not resolved in `run` scripts

## Summary

Workflows using `uses: https://data.forgejo.org/actions/cache@v4` (full URL action references) fail, and `${{ secrets.* }}` expressions in composite action `run:` scripts are not resolved before being passed to the shell. Both bugs are in the EphemeralAct Rust re-implementation.

## Reproduction

Run the publish workflow:

```bash
ephemeral_act run --workflow .forgejo/workflows/publish-staging.yml --job publish \
  --event pull_request --input mode=staging --input dev-counter=001 \
  --secret CRATES_IO_STAGING_TOKEN=test-staging-token
```

## Observed behavior

The cache steps are skipped:
```
[skipped] remote action not supported: https://data.forgejo.org/actions/cache@v4
```

The Rust toolchain installs correctly, but then the publish composite action fails:
```
Step './.forgejo/actions/publish' (composite): failed (exit code: 1)
bash: line 2: ${{ secrets.CRATES_IO_STAGING_TOKEN }}: bad substitution
```

## Root cause — two related bugs

### Bug 1: `${{ secrets.* }}` and other expressions not resolved in `run:` scripts

**File:** `src/core/services/step_runner_service.rs`

`run_shell_command()` passes the raw `cmd` string directly to `bash -c`:

```rust
fn run_shell_command(cmd: &str, ...) {
    let cmd_parts: Vec<String> = vec![shell.to_string(), "-c".to_string(), cmd.to_string()];
    container.exec(&cmd_parts, ...)
}
```

No expression resolution happens. The composite action `.forgejo/actions/publish/action.yml` has a `run:` step containing `${{ secrets.CRATES_IO_STAGING_TOKEN }}`. This literal string is sent to bash, which interprets `${{ ... }}` as a bash parameter expansion and fails with `bad substitution`.

`resolve_inputs()` only handles `${{ inputs.<name> }}` placeholders:

```rust
fn resolve_inputs(step: &Step, with: &HashMap<String, String>) -> Step {
    let resolve = |s: &str| -> String {
        let mut result = s.to_string();
        for (key, value) in with {
            let placeholder = format!("${{{{ inputs.{}. }}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    };
    // ... only replaces inputs.X, NOT secrets.X, github.X, env.X, etc.
}
```

### Bug 2: Secrets from CLI are never wired into the container environment or `EvalContext`

**File:** `src/core/services/run_act_service.rs` — `execute_run()`

Secrets are parsed from CLI args and stored in `ActRunConfig.secrets`, but `execute_run()` never reads `config.secrets()` to populate the container environment or construct an `EvalContext` with the secret values.

**File:** `src/core/expression/context.rs` — `EvalContext`

`EvalContext` already has a `secrets: Value` field and `get("secrets")` accessor. The expression evaluator (`src/core/expression/evaluator.rs`) can resolve `secrets.TOKEN` expressions. But `EvalContext` is never constructed or used in the execution pipeline — it exists but is dead code.

**File:** `src/core/value_objects/secret.rs` — `Secret`

The `Secret` type is stored in `ActRunConfig` but never consumed downstream.

## How to fix

### Fix 1: Resolve `${{ ... }}` expressions in `run:` scripts before passing to the shell

In `StepRunnerService::run_shell_command()`, resolve `${{ ... }}` expressions in `cmd` before constructing the `cmd_parts`. The resolution needs to:

1. Build an `EvalContext` populated from:
   - Container environment variables → `env`
   - `config.secrets()` → `secrets`
   - `config.inputs()` → `inputs`
   - `config.event()` → `github.event`
2. Parse `${{ ... }}` expressions in the command string (use the existing `Expr` parser from `src/core/expression/parser.rs`)
3. Evaluate each expression against the `EvalContext`
4. Substitute the evaluated values back into the command string

A pragmatic first step: extract all `${{ ... }}` placeholders from the command string, evaluate each using the expression evaluator, and replace them with the stringified results. For simple `secrets.X` values, also inject them as environment variables into the container so bash can reference `$SECRET_NAME` directly.

### Fix 2: Wire secrets into the container environment and `EvalContext`

In `run_act_service.rs` `execute_run()`:

```rust
// After building container_env from workflow/job env:
for secret in config.secrets() {
    // Parse "KEY=VALUE" format from ActRunConfig
    container_env.insert(secret.name().into(), secret.value().into());
}
```

And construct an `EvalContext` with the secrets populated:

```rust
let mut secrets_map = serde_json::Map::new();
for secret in config.secrets() {
    secrets_map.insert(secret.name().into(), Value::String(secret.value().into()));
}
let mut ctx = EvalContext::new();
ctx.secrets = Value::Object(secrets_map);
```

Then pass `ctx` to `StepRunnerService::execute()` so `resolve_inputs` (or a new expression resolver) can evaluate `${{ secrets.X }}`.

### Fix 3 (optional): Handle full-URL `uses:` references explicitly

`uses: https://data.forgejo.org/actions/cache@v4` currently works because `run_action()` falls into the `else` branch and skips it as `[skipped] remote action not supported`. This is acceptable for now, but if download support is added later, the URL format needs to be parsed to extract `{org}/{repo}` and `@ref`. The parser in `src/core/expression/parser.rs` or a new `ActionRef` type could handle `https://` prefixes by stripping the scheme and host to get `data.forgejo.org/actions/cache@v4`, then further parsing to `actions/cache@v4`.

## Files involved

| File | Role |
|------|------|
| `src/core/services/step_runner_service.rs` | `run_shell_command`, `run_action`, `run_local_action`, `resolve_inputs` |
| `src/core/services/run_act_service.rs` | `execute_run` — needs to wire secrets and EvalContext |
| `src/core/expression/context.rs` | `EvalContext` — exists but unused in execution |
| `src/core/expression/evaluator.rs` | Can evaluate `secrets.X` if context is populated |
| `src/core/expression/parser.rs` | Parses `${{ ... }}` expression strings |
| `src/core/value_objects/secret.rs` | `Secret` type — populated but never consumed |
| `src/core/value_objects/act_run_config.rs` | `ActRunConfig.secrets` — stored but never read by service |
| `src/core/ports/outbound/runner_context.rs` | `RunnerContext` — env map, needs secret injection |

## Verification

After the fix, running:

```bash
ephemeral_act run --workflow .forgejo/workflows/publish-staging.yml --job publish \
  --event pull_request --input mode=staging --input dev-counter=001 \
  --secret CRATES_IO_STAGING_TOKEN=test-staging-token
```

Should show the publish composite action's `run` script executing without `bad substitution`, and the cache steps can either be resolved (if download support is added) or continue to be skipped with a clear message.
