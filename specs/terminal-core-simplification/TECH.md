# Miniwarp Terminal-Core Reduction — Tech Spec
Product spec: `specs/terminal-core-simplification/PRODUCT.md`
Architecture reference: `ARCHITECTURE.md`
Execution checklist: `PLAN.md`

## Context

Miniwarp is intended to become a smaller terminal-first fork of Warp. The current repo already enables terminal-core mode in `app/src/features.rs:1-7`, but the architecture is still that of the broader Warp product. This spec is the technical contract for turning the current best-effort mode into a real reduced architecture.

## Current-state findings

### 1. Terminal-core mode exists, but only as a partial guardrail

- `app/src/features.rs:1-7` defines `TERMINAL_CORE_MODE` and `is_terminal_core_mode()`, and the mode is already hardcoded on.
- `app/src/lib.rs:181-284` defines `TERMINAL_CORE_DISABLED_FEATURE_FLAGS`.
- `app/src/lib.rs:2894-2898` only prunes feature flags at runtime.

This hides parts of the product, but it does not yet prevent broad non-core initialization or dependency retention.

### 2. Boot still pulls in major non-core surfaces

The app startup graph still initializes many surfaces that Miniwarp likely does not want:

- `app/src/lib.rs:1523-1579` still unconditionally initializes `workspace`, `terminal`, `editor`, `menu`, `tips`, `launch_configs`, `workflows`, theme choosers, `root_view`, `voltron`, `auth`, billing modals, env-var views, and SSH flows.
- Only some surfaces are currently gated, such as `ai`, `onboarding`, `ai::blocklist`, `drive`, `ai_assistant`, some environment/settings pieces, `coding_entrypoints`, and `code_review`.
- `app/src/lib.rs:1871-1883` still only conditionally skips `input_classifier` and `WorkflowAliases`, which means terminal-core mode is not the main architectural seam.

### 3. Workspace still assumes the full product

- `app/src/workspace/mod.rs:24-39` imports AI usage, skills, notebooks, settings-view sections, and agent telemetry concerns directly into the top-level module.
- `app/src/workspace/mod.rs:101-132` boots onboarding, launch modals, global search, right-panel UI, notebook wiring, code wiring, sync inputs, and LSP.

This means the workspace currently acts as a multi-product shell rather than a minimal terminal host.

### 4. Terminal still contains product add-ons

- `app/src/terminal/mod.rs:22-95` still includes `buy_credits_banner`, `share_block_modal`, `shared_session`, and `warpify` adjacent to core terminal modules.
- `app/src/terminal/mod.rs:124-127` still initializes `share_block_modal`.

So the terminal module is not yet a clean boundary for terminal-only behavior.

### 5. `cli_agent` is coupled to broader product types

- `app/src/terminal/cli_agent.rs:20-27` depends on `ai::agent`, `ai::blocklist`, `code`, `code_review`, telemetry, icons, and workspaces.
- `app/src/terminal/cli_agent_sessions/mod.rs:10-34` depends on `ai::blocklist::InputConfig` and `ai::agent::conversation::ConversationStatus`.
- `app/src/terminal/input/slash_commands/mod.rs:16-40` depends on `ai::blocklist`, `cloud_object`, `code_review`, workflows, settings, and workspace toasts/actions.

`cli_agent` is explicitly in scope to keep, so this coupling is the main technical risk.

### 6. Cargo and module surface are still broad

- `app/src/lib.rs:4-130` still registers major product areas such as `ai`, `billing`, `cloud_object`, `code_review`, `coding_entrypoints`, `drive`, `notebooks`, `pricing`, `projects`, `remote_server`, `resource_center`, `voltron`, `workspaces`, and `workflows`.
- `app/Cargo.toml:51-252` still brings in heavy dependencies that support the broader Warp product, including `ai`, `computer_use`, `firebase`, `warp_graphql`, `warp_server_client`, `voice_input`, `natural_language_detection`, `onboarding`, `warp-workflows`, `remote_server`, and others.

## Desired target

Miniwarp should converge on the architecture described in `ARCHITECTURE.md`:

- terminal rendering and PTY/session management;
- shell launch, history, completion, prompt handling, and terminal settings/themes;
- minimal workspace/tab/pane orchestration;
- minimal file-open/edit support for terminal-adjacent workflows;
- retained explicit `#`-triggered assistance if it remains in product scope;
- retained `cli_agent` detection, session tracking, rich input, and related telemetry hooks;
- no broad product surfaces whose value is primarily collaboration, cloud sync, notebooks, workflows, AI side panels, code review, or monetization/upsell.

## Relevant code

### Primary reduction seams

- `app/src/features.rs:1-7`
- `app/src/lib.rs:181-284`
- `app/src/lib.rs:1523-1579`
- `app/src/lib.rs:1871-1883`
- `app/src/lib.rs:2894-2898`
- `app/src/workspace/mod.rs:1-132`
- `app/src/terminal/mod.rs:1-127`
- `app/src/terminal/cli_agent.rs:1-220`
- `app/src/terminal/cli_agent_sessions/mod.rs:1-220`
- `app/src/terminal/input/slash_commands/mod.rs:1-120`
- `app/Cargo.toml:51-252`

### Representative non-core clusters likely to remove or isolate

- `app/src/code_review/mod.rs`
- `app/src/drive/mod.rs`
- `app/src/workflows/mod.rs`
- `app/src/notebooks/mod.rs`
- `app/src/resource_center/*`
- `app/src/billing/*`
- `app/src/pricing/*`

### Foundational areas likely to retain

- `app/src/terminal/*`
- `app/src/pane_group/*`
- `app/src/workspace/*` (trimmed)
- `app/src/editor/*` and `app/src/code/*` (trimmed to terminal-adjacent use)
- `app/src/settings/*`
- `app/src/themes/*`
- `crates/warp_terminal/*`
- `crates/warp_core/*`

## Proposed technical approach

### 1. Promote terminal-core mode into a real module boundary

Keep `app/src/features.rs` as the top-level switch for the transition, but move responsibility away from feature-flag filtering and into actual boot/module boundaries.

Concretely:

- use `app/src/lib.rs:1523-1579` as the primary boot graph to simplify;
- gate or delete non-core initializers there first;
- stop treating `TERMINAL_CORE_DISABLED_FEATURE_FLAGS` as the long-term product boundary;
- make startup order reflect the target architecture in `ARCHITECTURE.md`.

The end state is not “full Warp with lots of disabled flags.” The end state is a smaller app with fewer reachable modules.

### 2. Isolate `cli_agent` behind terminal-owned adapters before deleting dependencies

`cli_agent` is a keep area, but it currently relies on broad product-owned types. Before deleting upstream surfaces, introduce terminal-core-owned abstractions for:

- session lifecycle/status;
- rich-input open/closed/config state;
- optional review/comment metadata only if required for retained UX;
- any workspace callbacks needed by the terminal-hosted agent flows;
- retained telemetry payload mapping.

These adapters should become the only types `terminal` and `cli_agent` rely on for agent-facing behavior.

Immediate touchpoints:

- `app/src/terminal/cli_agent.rs`
- `app/src/terminal/cli_agent_sessions/mod.rs`
- `app/src/terminal/input/slash_commands/mod.rs`

Design rule:

- `terminal` and `cli_agent` may depend on terminal-core-owned types;
- they should not depend directly on large `ai::blocklist`, `code_review`, `workflows`, `drive`, or notebook-oriented modules.

### 3. Trim the terminal module to the true terminal surface

Refactor `app/src/terminal/mod.rs` so that retained terminal exports and initialization are clearly separated from removable legacy add-ons.

Expected actions:

- remove share/session/social surfaces unless a strict terminal-first case remains;
- remove buy-credits and other monetization UI embedded in terminal;
- keep terminal view/input/history/SSH/session/runtime pieces that are still in scope;
- keep `cli_agent` support attached to the terminal surface only through isolated adapters.

### 4. Reduce workspace to a terminal host

`workspace` should shrink to:

- windows, tabs, panes, and active session orchestration;
- retained menus/actions required for terminal use;
- settings access;
- lightweight terminal-adjacent file-open/edit support;
- essential toasts/modals that still apply to the reduced product.

`workspace` should stop hosting:

- notebook entry points;
- workflow product flows;
- global AI/product search panes;
- right-side agent/chat panels;
- launch flows and onboarding designed for the broader Warp product;
- settings sections that only configure deleted products.

### 5. Delete product clusters in logical groups

Delete or fully isolate, then delete, the clusters that do not belong to Miniwarp:

- notebooks;
- workflows;
- code review;
- drive/cloud-object-first UI;
- pricing, billing, rewards, referrals, changelog/reward marketing;
- resource-center and broad product-tour/onboarding surfaces;
- ambient/persistent AI panes outside retained explicit assistance.

Each group removal must include:

- its initialization hooks;
- workspace/menu/keybinding entry points;
- settings pages and flags;
- telemetry/events that only support the removed surface;
- Cargo/workspace dependencies once references are gone.

### 6. Prune dependencies after reference cleanup

Once references are removed from app code:

- remove no-longer-needed dependencies from `app/Cargo.toml`;
- remove or stop building crates that only support deleted product clusters;
- prefer crate/module deletion over indefinitely carrying dead code behind flags.

Dependency cleanup should happen after code references are cut so the remaining graph is obvious.

## Migration phases

### Phase A — Boot graph cleanup

Reduce `app/src/lib.rs` so terminal-core startup only initializes retained systems.

### Phase B — `cli_agent` adapter refactor

Break product-owned type dependencies before broad deletion.

### Phase C — Terminal and workspace reduction

Trim retained hosts (`terminal`, `workspace`, `root_view`) to their new responsibilities.

### Phase D — Product-cluster deletion

Remove major non-core feature groups one cluster at a time.

### Phase E — Cargo/workspace pruning

Delete now-unused dependencies and crates after source references are gone.

## Risks and mitigations

### Risk: `cli_agent` is more entangled than expected

Mitigation:

- do not delete underneath it first;
- isolate it behind adapters;
- verify one retained `cli_agent` flow after every major cut.

### Risk: workspace reduction sprawls into unrelated rewrites

Mitigation:

- keep the scope on terminal hosting, not UX redesign;
- remove one product cluster at a time;
- keep the app compiling after each cluster cut.

### Risk: dead product paths remain in menus/settings even after startup cleanup

Mitigation:

- require every deletion to include its commands, settings, bindings, and modals;
- use negative validation, not just compile success.

### Risk: dependency pruning happens too early

Mitigation:

- prune only after source references are removed;
- let compiler errors drive the remaining cleanup;
- prefer smaller grouped deletions over one giant Cargo sweep.

## Parallelization

Once the `cli_agent` adapter direction is established, work can proceed on mostly independent tracks:

1. **Boot/architecture track**
   - simplify startup and retained singleton registration.

2. **Terminal/CLI-agent track**
   - isolate `cli_agent`, trim terminal exports, remove terminal add-ons.

3. **Workspace/root-view track**
   - remove non-core panes, modals, bindings, and settings routing.

4. **Product-cluster deletion track**
   - remove notebooks, workflows, code review, drive, monetization, and upsell clusters.

5. **Dependency-pruning track**
   - remove now-unused dependencies and crates after code references are gone.

## Validation

### Core workflow validation

- build the app after each major phase;
- verify terminal creation, shell launch, pane splitting, tabs, command execution, history, completion, and theme/settings flows;
- verify retained terminal-adjacent file-open/edit paths;
- verify at least one retained `cli_agent` workflow end to end.

### Negative validation

- confirm removed surfaces no longer initialize in startup;
- confirm removed surfaces are absent from menus, settings, modals, and panels;
- confirm retained terminal code no longer imports deleted product modules;
- confirm removed dependencies are gone from `app/Cargo.toml` and the workspace graph.

### Documentation validation

- keep `PRODUCT.md`, `TECH.md`, `PLAN.md`, and `ARCHITECTURE.md` aligned with the implementation state;
- update the “current-state findings” section once the first major cleanup pass lands.

## Follow-ups

- When the first cleanup wave lands, rewrite the current-state section so it documents the reduced repo rather than the pre-reduction repo.
- If the retained explicit `#`-triggered assistance scope changes, update both `PRODUCT.md` and `ARCHITECTURE.md` in the same PR.
