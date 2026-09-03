# Using ephact

## Running a workflow

```sh
ephact run --repo /path/to/repo --workflow .github/workflows/ci.yml --event push
```

## Key flags

| Flag | Description | Default |
|------|-------------|---------|
| `--workflow FILE` | Workflow file to run (relative to the repo root) | *required* |
| `--event EVENT` | Event to simulate (`push`, `pull_request`, `workflow_dispatch`, `release`) | *required* |
| `--ref REF` | Git ref the event appears to have happened on | current HEAD |
| `--secret NAME=VALUE` | Inject a secret as `${{ secrets.NAME }}` (repeatable) | none |
| `--input NAME=VALUE` | Inject an input as `${{ inputs.NAME }}` (repeatable) | none |
| `--output FORMAT` | Run summary format: `human` or `json` | `human` |
| `--allow-real-container` | Use the real Docker/Podman adapter instead of the fake runtime | *false* |
| `--allow-real-fetcher` | Fetch actions from the forge instead of the local mirror | *false* |
| `--allow-network` | Allow outbound network in containers (requires `--allow-real-container`) | *false* |

Events are simulated from flags only: `ephact` does not inspect a real Git
repository to decide what happened.

## Safe by default

By default runs are side-effect-free: actions come from a local mirror,
containers are a fake runtime that blocks host writes and outbound traffic,
and secrets are never written to disk. Real containers, real fetchers, and
network access are strictly opt-in through the `--allow-*` flags above.
