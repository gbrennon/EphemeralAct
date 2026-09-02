# EphemeralAct Usage Guide

## Overview
EphemeralAct is a Rust re‑implementation of a GitHub Actions runner that runs workflows in an **ephemeral, side‑effect‑free** manner by default.  
All interactions with the outside world (network calls to fetch actions, container creation, file writes) go through injectable ports.  
In the default configuration those ports are backed by **in‑memory fakes** that record what would have happened without actually performing the operation.

## Default Behaviour – No Real-World Side Effects
- **Actions** are fetched from a local mirror or from the network only if the `GitActionFetcher` is given a real base URL; the test suite uses a local mirror that never contacts the real forge.
- **Containers** are created via the `ContainerRuntimePort`. **By default the fake runtime is used, which blocks write operations to the host filesystem and does not allow outbound network traffic unless explicitly enabled.** In production builds the real Docker/Podman adapters are used, but they only create containers inside the engine; they never push images, pull from private registries, or modify the host filesystem beyond the bind‑mounted workspace unless the user opts in with `--allow-real-container` and/or `--allow-network`.
- **Secrets and inputs** are resolved from environment variables or CLI flags and are never written to disk or leaked.
- **No external services** (e.g. artifact registries, deployment targets) are called unless the workflow itself explicitly calls them (e.g. an `npm publish` step). Those calls happen inside the container and can be intercepted by the container’s network stack if you wish to block them.
Because the default DI container wires the *fake* implementations (`SucceedingRuntime`, `MirroredActionFetcher`, `FixedImageMapper`, etc.) when running the test suite (or when `EPHEMERAL_ACT_USE_FAKE_CONTAINER=1` is set), **nothing touches the real world** unless you explicitly opt‑in.

## Making EphemeralAct Touch the Real World
If you need to run a workflow against real container engines or a real action registry, you can opt‑in via CLI flags (or by providing a custom DI container).  
The flags are deliberately opt‑in to keep the tool safe by default.

| Flag | Description | Default |
|------|-------------|---------|
| `--allow-real-container` | Use the real Docker or Podman adapter instead of the fake container runtime. | *false* (use fake) |
| `--allow-real-fetcher`   | Use the real `GitActionFetcher` that talks to the configured forge (GitHub.com, Forgejo, etc.) instead of the local mirror. | *false* (use mirror) |
| `--allow-network`        | Allow containers to make outbound network requests (e.g. to `npm registry`, `docker hub`). This flag is only respected when `--allow-real-container` is also set; fake containers always drop network traffic. | *false* |
| `--secret NAME=VALUE`    | Provide a secret value that will be injected as `${{ secrets.NAME }}`. Can be repeated. | *none* |
| `--input NAME=VALUE`     | Provide an input value that will be injected as `${{ inputs.NAME }}`. Can be repeated. | *none* |
| `--workflow FILE`        | Path (relative to the repo root) of the workflow file to run. | *required* |
| `--event EVENT`          | GitHub event that should trigger the workflow (e.g. `push`, `pull_request`, `workflow_dispatch`, `release`). | *required* |
| `--ref REF`              | Git ref (branch, tag, or SHA) that the event should appear to have happened on. Defaults to the repo’s current HEAD. | *current HEAD* |
| `--output FORMAT`        | Output format for the run summary: `human` (default) or `json`. | `human` |
| `--help`                 | Show help message. | — |

**Example – Real container, real fetcher, but still no network:**

```bash
ephemeral_act run \
  --repo /path/to/myrepo \
  --workflow .github/workflows/ci.yml \
  --event push \
  --ref main \
  --allow-real-container \
  --allow-real-fetcher \
  --secret GITHUB_TOKEN=ghp_... \
  --output json > run.json
```

## Simulating Specific Git Events
EphemeralAct does **not** interact with a real Git repository to determine the event; it purely simulates the event payload based on the flags you provide.

### Simulating a `push` event
```bash
ephemeral_act run \
  --repo /path/to/myrepo \
  --workflow .github/workflows/push.yml \
  --event push \
  --ref refs/heads/main
```

### Simulating a `pull_request` event
```bash
ephemeral_act run \
  --repo /path/to/myrepo \
  --workflow .github/workflows/pr.yml \
  --event pull_request \
  --ref refs/pull/42/merge   # the ref that represents the merge commit
```
You can also add `--secret GITHUB_TOKEN=...` if the workflow needs it.

### Simulating a `release` event (e.g. tag push)
```bash
ephemeral_act run \
  --repo /path/to/myrepo \
  --workflow .github/workflows/release.yml \
  --event release \
  --ref refs/tags/v1.2.3
```

### Simulating a `workflow_dispatch` (manual trigger) with inputs
```bash
ephemeral_act run \
  --repo /path/to/myrepo \
  --workflow .github/workflows/dispatch.yml \
  --event workflow_dispatch \
  --ref refs/heads/main \
  --input version=2.0.0 \
  --input environment=staging
```

## Making a Workflow Run Against the Current Worktree
By default EphemeralAct **bind‑mounts** the repository directory you give with `--repo` into the container at `/workspace`.  
Any changes you have in the working tree (uncommitted edits, untracked files) will be seen inside the container exactly as they appear on disk.  
If you want to test against a specific commit or tag without affecting your working copy, you can:

1. Create a temporary worktree or clone:
   ```bash
   git worktree add -b test-run ../myrepo-test
   cd ../myrepo-test
   git checkout <desired-commit>
   ```
2. Run EphemeralAct pointing at that worktree:
   ```bash
   ephemeral_act run --repo $(pwd) --workflow .github/workflows/ci.yml --event push --ref <desired-commit>
   ```

## Ensuring No Real‑World Modifications
To double‑check that a run did *not* perform any undesired side effects, you can:

- Run with `--allow-real-container` **false** (the default) and inspect the `ContainerActivity` fake if you are using the test harness. In a normal binary you can replace the container runtime with a fake by setting the environment variable `EPHEMERAL_ACT_USE_FAKE_CONTAINER=1` (the DI container reads this and swaps in the fake).
- Use `--allow-network=false` to guarantee containers cannot reach external registries.
- Examine the produced JSON report (`--output json`) – it lists every command that was executed inside containers. If you see a command like `curl https://registry.npmjs.org/...` you know network was allowed; otherwise, with the flag unset, such commands will fail inside the container (or be dropped by the fake).

## Documented Example: Simulating a Merge Trigger
Suppose you have a workflow `.github/workflows/merge.yml` that should run when a pull request is merged. To simulate that:

```bash
# 1. Ensure you have the branch you want to merge checked out locally
git checkout feature/add-login
git pull origin feature/add-login   # update if needed

# 2. Run the workflow as a pull_request event with the merge ref
ephemeral_act run \
  --repo $(pwd) \
  --workflow .github/workflows/merge.yml \
  --event pull_request \
  --ref refs/pull/123/merge   # 123 is the PR number you want to pretend
  --secret GITHUB_TOKEN=dummy   # if the workflow needs it
  --output json > merge-run.json
```

The `--ref` flag tells EphemeralAct to populate the `github.ref` and `github.sha` contexts as if the merge commit at the tip of the PR were the triggering event. No actual merge is performed on your local repository; the simulation is purely contextual.

## Summary
- **Default**: EphemeralAct runs in a completely isolated, fake‑backed environment – zero side effects.
- **Opt‑in real world**: Use `--allow-real-container`, `--allow-real-fetcher`, and/or `--allow-network` to enable real Docker/Podman, real action fetching, and real network traffic.
- **CLI driven**: All necessary inputs (workflow, event, ref, secrets, inputs) are supplied via flags, making the tool scriptable and safe for CI.
- **Event simulation**: Use `--event` and `--ref` to emulate any GitHub event (including pull request merges) without touching the actual repository.

Keep this file (`USAGE.md`) in the repository’s root so that contributors and users have a clear reference on how to run EphemeralAct safely and how to opt‑in to real‑world interactions when explicitly desired.