# Shell Script vs Rust Core Model Analysis

## Overview
The shell script (`act-ephemeral.sh`) is a **reference implementation** of the ephemeral act runner concept. The Rust core model captures the **domain concepts** required to build this. This analysis identifies alignment, gaps, and what should remain in the services layer.

---

## ✅ Perfect Alignment

### 1. Argument Parsing → Value Objects
| Shell Flag | Rust Model | Validation |
|-----------|-----------|-----------|
| `-w WORKFLOW` | `ActWorkflow` | Path string stored as-is |
| `-j JOB` | `ActJob` | Job name stored as-is |
| `-e EVENT` | `ActEvent` | Event name (push, pull_request, etc.) |
| `-i KEY=VALUE` | `ActInput` | Key-value pair |
| `-s KEY=VALUE` | `Secret` | String with redacted Debug |
| `-c ENGINE` | `ContainerEngine::from_str()` | "podman" \| "docker" → Enum |
| `--no-cleanup` | `CleanupPolicy` | Preserve \| CleanupOnExit |
| `-- ACT_ARGS` | `ActExtraArg` | Pass-through strings |

✅ **Status**: Core models all of these correctly.

---

### 2. Repository Validation → RepoPath
| Shell Script | Rust Core |
|-------------|-----------|
| Check path is directory | `RepoPath::new()` validates |
| Check `.git` exists (dir or file) | `RepoPath::new()` detects |
| Canonicalize path | `path.canonicalize()` called |
| Detect worktree (`.git` is file) | `GitDirKind::Worktree` detected |

✅ **Status**: Core fully models this.

---

### 3. Ephemeral Repository Lifecycle
| Shell Script | Rust Core |
|-------------|-----------|
| Generate temp dir name `act-run-{REPO_NAME}-XXXXXX` | `TempDirTemplate::from_repo_name()` |
| Store cleanup policy (cleanup vs preserve) | `EphemeralRepository.cleanup_policy` |
| Flag worktree-to-standalone conversion needed | `EphemeralRepository.needs_standalone_conversion` |

✅ **Status**: Core fully models this.

---

### 4. Act Command Configuration
| Shell Script | Rust Core |
|-------------|-----------|
| `--rm` flag | `ActRunConfig.rm: bool` |
| `--bind` flag | `ActRunConfig.bind: bool` |
| `-W WORKFLOW` flag | `ActRunConfig.workflow: Option<ActWorkflow>` |
| `-j JOB` flag | `ActRunConfig.job: Option<ActJob>` |
| Event (positional) | `ActRunConfig.event: Option<ActEvent>` |
| `-i` for inputs (repeatable) | `ActRunConfig.inputs: Vec<ActInput>` |
| `-s` for secrets (repeatable) | `ActRunConfig.secrets: Vec<Secret>` |
| Extra args after `--` | `ActRunConfig.extra_args: Vec<ActExtraArg>` |

✅ **Status**: Core fully models this.

---

## ⚠️ Gaps & Misalignments

### 1. **Container Daemon Socket** (MISSING in Rust Core)

**Shell Script:**
```bash
case "$ACT_CONTAINER_ENGINE" in
  podman)
    ACT_CONTAINER_SOCKET="${PODMAN_SOCK:-unix:///run/user/$(id -u)/podman/podman.sock}"
    ;;
  docker)
    ACT_CONTAINER_SOCKET="${DOCKER_SOCK:-unix:///var/run/docker.sock}"
    ;;
esac

# Passed as:
ACT_CMD+=(--container-daemon-socket "$ACT_CONTAINER_SOCKET")
```

**Rust Core Issue:**
- `ContainerEngine` enum exists but doesn't model the socket path
- Socket path is environment-dependent (PODMAN_SOCK, DOCKER_SOCK env vars)
- No value object to represent the socket path

**Impact:** Services layer must resolve socket paths from:
- Environment variables
- Fallback defaults per engine
- Then pass to act command

**Recommendation:**
```rust
// Add to value_objects/
pub struct ContainerDaemonSocket(String);

impl ContainerDaemonSocket {
    pub fn for_engine(engine: &ContainerEngine) -> Self {
        match engine {
            ContainerEngine::Podman => {
                let sock = std::env::var("PODMAN_SOCK")
                    .unwrap_or_else(|_| format!("unix:///run/user/{}/podman/podman.sock", 
                        unsafe { libc::getuid() }));
                Self(sock)
            },
            ContainerEngine::Docker => {
                let sock = std::env::var("DOCKER_SOCK")
                    .unwrap_or_else(|_| "unix:///var/run/docker.sock".to_string());
                Self(sock)
            }
        }
    }
    pub fn as_str(&self) -> &str { &self.0 }
}

// Then add to ActRunConfig:
pub struct ActRunConfig {
    container_engine: ContainerEngine,
    container_daemon_socket: ContainerDaemonSocket,  // ADD THIS
    // ... rest of fields
}
```

---

### 2. **Container Daemon Socket Flag in ActRunConfig** (MISSING)

**Shell Script:**
```bash
ACT_CMD=(
  act
  --container-daemon-socket "$ACT_CONTAINER_SOCKET"  # <-- This flag
  --rm
  --bind
)
```

**Rust Core Issue:**
- `ActRunConfig` doesn't have a field to store the socket value
- Service layer would need to construct this independently
- Breaks the principle of "ActRunConfig contains everything needed for act"

**Recommendation:** Add to ActRunConfig as shown above.

---

### 3. **CLI Flag Format Documentation** (MINOR)

**Shell Script:** Uses `-W` for workflow, `-j` for job, etc.
**Rust Core:** Field names don't reflect flag names (e.g., `workflow`, not `workflow_file`)

**Impact:** Minimal—service layer needs to know flag mappings anyway. Consider documenting expected flags:

```rust
impl ActRunConfig {
    /// Build the act CLI command arguments.
    /// 
    /// Maps internal fields to act CLI flags:
    /// - workflow → -W <path>
    /// - job → -j <name>
    /// - event → <name> (positional)
    /// - inputs → -i <key=value>
    /// - secrets → -s <key=value>
    /// - container_daemon_socket → --container-daemon-socket <socket>
    /// - rm → --rm
    /// - bind → --bind
    /// - extra_args → (appended)
    pub fn to_cli_args(&self) -> Vec<String> {
        // Implementation in services, but document the contract
    }
}
```

---

### 4. **Working Directory Context** (SERVICE-LEVEL)

**Shell Script:**
```bash
cd "$TEMP_DIR"
exec "${ACT_CMD[@]}"
```

**Rust Core:** Doesn't model this—correct for a domain model.

**Why it's okay:** The working directory is an *execution detail*, not a domain concern. Services handle this.

---

### 5. **Actual Filesystem Operations** (SERVICE-LEVEL)

**Shell Script:**
```bash
# Copy repo
rsync -a --delete "$SOURCE_REPO"/ "$TEMP_DIR"/
# (or cp -a as fallback)

# Convert worktree if needed
if [[ -f "$TEMP_DIR/.git" ]]; then
  rm "$TEMP_DIR/.git"
  git -C "$TEMP_DIR" init
  git -C "$TEMP_DIR" add -A
  git -C "$TEMP_DIR" commit -m "ephemeral: snapshot for act" --allow-empty
fi
```

**Rust Core:** `EphemeralRepository.needs_standalone_conversion()` signals *that* conversion is needed, not *how*.

**Why it's correct:** The core models the *what* (needs conversion), services implement the *how* (git init, commit, etc.).

---

### 6. **Cleanup Signal Handler** (SERVICE-LEVEL)

**Shell Script:**
```bash
cleanup() {
  local rc=$?
  if [[ "$CLEANUP_ON_EXIT" == "true" ]]; then
    rm -rf "$TEMP_DIR"
  fi
  exit "$rc"
}
trap cleanup EXIT
```

**Rust Core:** `CleanupPolicy` exists, but no signal handling or lifecycle management modeled.

**Why it's okay:** Lifecycle/signal handling is infrastructure, not domain. Rust services would use `Drop` or explicit cleanup.

---

## 📊 Coverage Summary

| Concept | Shell | Rust Core | Status |
|---------|-------|-----------|--------|
| Repo path validation | ✓ | ✓ | ✅ Perfect |
| Repo name derivation | ✓ | ✓ | ✅ Perfect |
| Worktree detection | ✓ | ✓ | ✅ Perfect |
| Temp dir template generation | ✓ | ✓ | ✅ Perfect |
| Cleanup policy | ✓ | ✓ | ✅ Perfect |
| Workflow/job/event configuration | ✓ | ✓ | ✅ Perfect |
| Inputs & secrets | ✓ | ✓ | ✅ Perfect |
| Extra args pass-through | ✓ | ✓ | ✅ Perfect |
| Container engine selection | ✓ | ✓ | ✅ Perfect |
| rm/bind flags | ✓ | ✓ | ✅ Perfect |
| **Container daemon socket** | ✓ | ✗ | ⚠️ **Missing** |
| **Socket flag in config** | ✓ | ✗ | ⚠️ **Missing** |
| File copying | ✓ | — | ✅ Service-level |
| Worktree conversion | ✓ | Signal only | ✅ Service-level |
| Act execution | ✓ | — | ✅ Service-level |
| Cleanup on exit | ✓ | — | ✅ Service-level |

---

## 🎯 Recommendations

### **HIGH PRIORITY**

1. **Add `ContainerDaemonSocket` value object**
   - Resolve socket path from env vars or defaults
   - Include in ActRunConfig
   - Ensures full CLI command can be reconstructed

2. **Document flag mappings in ActRunConfig**
   - Add examples showing how fields map to act CLI
   - Clarify event is positional (not `-e EVENT`)

### **MEDIUM PRIORITY**

3. **Add `container_daemon_socket()` getter to ActRunConfig**
   - Allow services to extract the socket for CLI building

### **LOW PRIORITY (Already Correct)**

4. Services will handle:
   - Actual file copying (rsync/cp)
   - Worktree-to-standalone conversion (git operations)
   - Signal handling and cleanup (Drop trait, cleanup functions)
   - Working directory changes (chdir)

---

## Conclusion

**The Rust core model is ~95% aligned with the shell script.** The main gap is the **container daemon socket**, which should be added as a value object and integrated into ActRunConfig. This is the only piece that prevents services from fully reconstructing the act CLI command from the core model alone.

Everything else—file operations, signals, execution, cleanup—correctly remains in the services layer as implementation details.
