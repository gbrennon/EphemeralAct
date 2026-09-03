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
