# Architecture

`ephact` is organised as a hexagon with four layers under `src/`:

| Layer | Responsibility |
|---|---|
| `domain` | Entities, value objects, workflow and expression model, planner, domain events and errors |
| `application` | Inbound/outbound ports, DTOs, one service per use case |
| `infrastructure` | Adapters: action fetching, container runtime (Bollard), image handling, runners, workflow loading, events, DI |
| `presentation` | CLI and the composition root that wires the container |

The dependency rule points inwards: `domain` depends on nothing,
`application` only on `domain`, and the outer layers implement the ports the
inner ones declare. All interactions with the outside world - fetching
actions, creating containers, writing files - go through injectable ports,
which keeps the default configuration side-effect-free.

The test suite mirrors the layers: `tests/application`,
`tests/infrastructure`, `tests/presentation`, and `tests/e2e`.

## Application Services Orchestration & Coordination

Application services coordinate decisions made by the domain, but **never depend directly on one another or on inbound ports**. All level-to-level coordination flows strictly through commands published via `CommandBusPort`. Infrastructure command handlers receive these commands and invoke the corresponding coordination service through its inbound port (`workflow -> job -> step -> action`).

```text
                        ┌────────────────────────┐
                        │ Presentation: CLI / Run│
                        └───────────┬────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    │ --all-workflows               │ single workflow
                    ▼                               ▼
        ┌───────────────────────┐       ┌───────────────────────┐
        │ RunAllWorkflowsService│       │   RunWorkflowService  │
        └───────────┬───────────┘       └───────────┬───────────┘
                    │ (1) dispatch_workflow         │ (1) dispatch_workflow
                    └───────────────┬───────────────┘
                                    ▼
╔═══════════════════════════════════════════════════════════════════════════╗
║                                                                           ║
║                        SINGLE SHARED COMMAND BUS                          ║
║                             (CommandBusPort)                              ║
║                                                                           ║
║  (1) ExecuteWorkflowCommand                                               ║
║       │                                                                   ║
║       ▼                                                                   ║
║      WorkflowCommandHandler ──► ExecuteWorkflowService                    ║
║                                        │                                  ║
║  (2) ExecuteJobCommand                 │ (dispatches)                     ║
║       │ ◄──────────────────────────────┘                                  ║
║       ▼                                                                   ║
║      JobCommandHandler      ──► ExecuteJobService                         ║
║                                        │                                  ║
║  (3) ExecuteStepCommand                │ (dispatches)                     ║
║       │ ◄──────────────────────────────┘                                  ║
║       ▼                                                                   ║
║      StepCommandHandler     ──► ExecuteStepService                        ║
║                                        │                                  ║
║  (4) ExecuteActionCommand              │ (dispatches)                     ║
║       │ ◄──────────────────────────────┘                                  ║
║       ▼                                                                   ║
║      ActionCommandHandler   ──► ExecuteActionService (Node / Composite)  ║
║                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════╝
```

### Coordination Flow Summary

1. **Entrypoints**: The CLI invokes `RunWorkflowPort` or `RunAllWorkflowsPort`. The services load workflow YAML from `WorkflowSourcePort` and dispatch `ExecuteWorkflowCommand` over `CommandBusPort`.
2. **Workflow Execution**: `WorkflowCommandHandler` parses the run configuration and context, calling `ExecuteWorkflowService` via `ExecuteWorkflowPort`. The service plans stage runs using the domain `Planner` and dispatches `ExecuteJobCommand` for each job.
3. **Job Execution**: `JobCommandHandler` invokes `ExecuteJobService` via `ExecuteJobPort`. The service sets up the job container via `PrepareJobContainerPort`, prepares environment variables, and dispatches `ExecuteStepCommand` for each step.
4. **Step Execution**: `StepCommandHandler` delegates to `ExecuteStepService` via `ExecuteStepPort`. Steps are evaluated and interpolated with `StepInterpolator`. If the step executes a shell command (`run:`), it runs via `RunShellStepPort`. If the step references an action (`uses:`), it dispatches `ExecuteActionCommand`.
5. **Action Execution**: `ActionCommandHandler` delegates to `ExecuteActionService` via `ExecuteActionPort`, resolving inputs, fetching actions, and executing either composite steps (which may recursively dispatch nested action commands) or Node.js actions within the job container.
6. **Cleanup**: Upon workflow completion, `RunWorkflowService` publishes `DomainEvent::ActRunCompleted` over `EventBusPort`, triggering `ContainerCleanupHandler` to stop and clean up containers.
