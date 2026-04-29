# Terminal-Core Simplification — Tech Spec

## Context

This document is the working contract for reducing Miniwarp to the core terminal workflows while preserving `cli_agent` support. There is no sibling `PRODUCT.md` yet, so this spec records both the current-state findings and the execution plan.

### Current state

Miniwarp already has a hardcoded terminal-core switch, but the codebase is only partially adapted to it:

- `app/src/features.rs:1-7` defines `TERMINAL_CORE_MODE` and `is_terminal_core_mode()`, and the constant is already set to `true`.
- `app/src/lib.rs:181-284` defines `TERMINAL_CORE_DISABLED_FEATURE_FLAGS`, which disables a large set of AI, collaboration, workflow, cloud, and code-review feature flags when terminal-core mode is active.
- `app/src/lib.rs:2894-2898` applies that filtering at runtime.

That guardrail is useful, but it is not yet the architectural boundary. The app still initializes substantial non-core surfaces unconditionally:

- `app/src/lib.rs:1523-1579` shows only some subsystems gated behind `!is_terminal_core_mode()`. `ai`, `onboarding`, `ai::blocklist`, `drive`, `ai_assistant`, `coding_entrypoints`, and `code_review` are guarded, but `workspace`, `terminal`, `editor`, `tips`, `workflows`, `themes`, `root_view`, `voltron`, `auth`, `billing` modals, env-var views, and SSH flows still initialize unconditionally.
- `app/src/lib.rs:1871-1883` still conditionally initializes only `input_classifier` and `WorkflowAliases`, which means many non-core dependencies survive even after the feature-flag filtering.

The workspace layer also still assumes the full product:

- `app/src/workspace/mod.rs:24-39` imports AI usage, notebooks, settings UI, and agent telemetry concerns directly into the top-level workspace module.
- `app/src/workspace/mod.rs:101-132` initializes `hoa_onboarding`, multiple launch modals, global search, right-panel UI, settings actions, `notebooks::init`, `code::init`, and `lsp::init` as part of normal workspace boot.

The terminal layer is closer to the desired target, but it still contains product surfaces that are not part of a minimal terminal:

- `app/src/terminal/mod.rs:22-95` includes core terminal modules alongside `buy_credits_banner`, `share_block_modal`, `shared_session`, and `warpify`.
- `app/src/terminal/mod.rs:124-127` initializes `share_block_modal` as part of terminal boot.

The `cli_agent` path is important but not yet isolated from non-core modules:

- `app/src/terminal/cli_agent.rs:20-27` depends on `ai::agent`, `ai::blocklist`, `code`, `code_review`, telemetry, icons, and workspaces.
- `app/src/terminal/cli_agent_sessions/mod.rs:10-34` depends on `ai::blocklist::InputConfig` and `ai::agent::conversation::ConversationStatus`.
- `app/src/terminal/input/slash_commands/mod.rs:16-40` couples slash-command handling to `ai::blocklist`, `cloud_object`, `code_review`, `workflows`, and workspace toasts/actions.

The repo surface confirms that this is still the full Warp product, not yet a trimmed terminal distribution:

- `app/src/lib.rs:4-130` registers major product modules including `ai`, `billing`, `cloud_object`, `code_review`, `coding_entrypoints`, `drive`, `notebooks`, `pricing`, `projects`, `remote_server`, `workspaces`, `ai_assistant`, `resource_center`, `workflows`, and `workspace`.
- `app/Cargo.toml:51-240` still pulls in heavy non-core dependencies such as `ai`, `computer_use`, `firebase`, `warp_graphql`, `warp_server_client`, `voice_input`, `natural_language_detection`, and `onboarding`.

### Relevant code

Core terminal-first files to preserve and reshape around:

- `app/src/features.rs:1-7` — terminal-core mode switch.
- `app/src/lib.rs:181-284` — current feature-flag disable list.
- `app/src/lib.rs:1523-1579` — top-level app initialization and the current terminal-core gating seams.
- `app/src/lib.rs:2894-2898` — runtime feature pruning.
- `app/src/terminal/mod.rs:22-127` — terminal module surface and terminal initialization.
- `app/src/terminal/cli_agent.rs:1-220` — CLI agent detection/configuration and an important dependency hotspot.
- `app/src/terminal/cli_agent_sessions/mod.rs:1-220` — CLI agent session tracking and another dependency hotspot.
- `app/src/terminal/input/slash_commands/mod.rs:16-40` — slash-command coupling to AI/workflow/code-review systems.
- `app/src/workspace/mod.rs:1-132` — workspace bootstrap and full-product assumptions.

Representative non-core product surfaces currently compiled into the app:

- `app/src/code_review/mod.rs:1-82` — code review UI, bindings, and terminal coupling.
- `app/src/drive/mod.rs:1-180` — Warp Drive object UI and workflow/notebook integration.
- `app/src/workflows/mod.rs:1-180` — workflow model and cloud/workspace integration.
- `app/src/notebooks/mod.rs:1-180` — notebook model and cloud sync integration.
- `app/Cargo.toml:51-240` — broad dependency surface that prevents the binary from becoming a narrowly scoped terminal app.
- `crates/warp_terminal/Cargo.toml:8-42` and `crates/warp_core/Cargo.toml:8-79` — foundational crates that should remain in the reduced architecture.

## Proposed changes

### 1. Establish the target boundary explicitly

The target product is:

- terminal rendering, PTY/session management, shell launch, completion, history, settings/theme support, pane layout, and minimal workspace/tab management;
- minimal editor/file-open support where it directly improves terminal workflows;
- preserved `cli_agent` detection, session tracking, rich input, and related telemetry;
- no product surfaces whose primary value comes from cloud sync, collaboration, AI chat panes, notebooks, workflows, referrals/rewards, billing/pricing, resource-center upsell, or shared-session/social features.

This spec should remain the source of truth for what belongs inside or outside that boundary until the cleanup is complete.

### 2. Convert terminal-core mode from a flag filter into a module boundary

The existing `TERMINAL_CORE_MODE` support should become the first-class architecture seam rather than a best-effort UI hide:

- keep `app/src/features.rs:1-7` as the entrypoint for the mode;
- treat `app/src/lib.rs:1523-1579` as the main boot graph to simplify;
- move non-core initialization behind explicit terminal-core exclusions until those modules are fully deleted;
- stop relying on feature flags alone to “hide” features that still compile, initialize models, and pull in dependencies.

The immediate goal is not perfection in one PR; it is to make every removal step compile cleanly while shrinking the reachable graph.

### 3. Preserve `cli_agent` by carving out a stable adapter layer first

`cli_agent` must remain supported, but the current implementation is coupled to AI/chat and code-review types. Before deleting broad product surfaces, introduce a narrow set of terminal-core-owned abstractions for:

- session status;
- rich-input mode/configuration;
- review-comment metadata only if still needed for terminal-hosted agent UX;
- workspace hooks used by `cli_agent`.

The first refactor step should target:

- `app/src/terminal/cli_agent.rs:20-27`;
- `app/src/terminal/cli_agent_sessions/mod.rs:10-34`;
- `app/src/terminal/input/slash_commands/mod.rs:16-40`.

The design rule is that `terminal` and `cli_agent` should depend only on terminal-core-owned interfaces, not on large `ai::blocklist`, `code_review`, `drive`, or notebook/workflow modules.

### 4. Remove non-core bootstrapping in phases

#### Phase A: obvious product/UI deletions

Delete or fully gate the surfaces that do not belong to terminal-first workflows and have little or no justification for remaining:

- referrals/rewards, resource-center promotion, pricing, billing, changelog/reward marketing surfaces;
- onboarding and launch-modal product tours that are not necessary for terminal use;
- notebooks, workflows, coding entrypoints, AI assistant side panels, and similar document/workflow products;
- shared-session and share-block UI from the terminal module unless a narrow terminal-core use case remains.

Primary touchpoints:

- `app/src/lib.rs:1523-1579`
- `app/src/workspace/mod.rs:101-132`
- `app/src/terminal/mod.rs:22-127`
- `app/src/code_review/mod.rs:1-82`
- `app/src/drive/mod.rs:1-180`
- `app/src/workflows/mod.rs:1-180`
- `app/src/notebooks/mod.rs:1-180`

#### Phase B: workspace simplification

Reduce `workspace` to terminal/session orchestration, pane management, basic settings access, and file-opening support:

- remove assumptions that every window can open notebooks, launch modals, global AI search, right-side agent panels, or settings subsections that only configure deleted products;
- prune toolbar items, actions, and tab/panel registrations that only exist for non-core product surfaces;
- keep only the minimal workspace affordances required for terminal tabs, panes, and terminal-adjacent editing.

This is primarily a `workspace` and `root_view` reduction, not a `terminal` rewrite.

#### Phase C: dependency and crate pruning

Once app/module references are removed, prune the workspace dependency graph:

- remove unnecessary entries from `app/Cargo.toml:51-240`;
- remove or stop building crates that only support deleted product lines;
- prefer deleting entire crates/modules over keeping dead feature-flagged code indefinitely.

This phase should come after reference cleanup so it is obvious which crates are still truly required.

### 5. Prefer deletion over permanent hiding

The long-term goal is not “terminal core mode” as a runtime fork inside the same product surface. The goal is a smaller, easier-to-maintain terminal app. For any module that has no remaining role after the target boundary is applied:

- first gate it if that is the safest incremental step;
- then delete the module, bindings, actions, settings, models, and dependencies that are left behind.

The repo should converge toward fewer modules and fewer workspace members, not a growing list of disabled flags.

### 6. Keep only minimal cross-cutting systems

These areas should be retained, but possibly simplified:

- `terminal`, `pane_group`, and minimal `workspace`;
- `settings`, `themes`, `appearance`, and terminal-specific keybindings;
- `editor` and `code` only where necessary for opening/editing files from terminal flows;
- `server` and telemetry support only where it is required for app runtime, terminal services, or `cli_agent`;
- `warp_terminal`, `warp_core`, `warp_features`, `warp_completer`, `warp_cli`, `warpui`, and related foundation crates.

Anything above that baseline needs a specific terminal-first justification to remain.

## Risks and mitigations

### `cli_agent` is more entangled than it looks

Risk:

- `cli_agent` currently touches AI session concepts, blocklist input config, code-review metadata, telemetry, and workspace state.

Mitigation:

- do not delete those dependencies directly from underneath `cli_agent`;
- first introduce terminal-core-local types/adapters, then migrate the implementation to them, then delete the old dependencies.

### Terminal module still contains collaboration/product affordances

Risk:

- `app/src/terminal/mod.rs:35,73-82,94` shows product surfaces embedded directly into the terminal module, so naïve deletion can break initialization or exports.

Mitigation:

- split terminal-core exports from legacy add-ons first;
- make terminal initialization only register the terminal-core subset before deleting the rest.

### Workspace cleanup can sprawl

Risk:

- `workspace` is currently the host for many unrelated product lines, so a single giant rewrite would be hard to validate.

Mitigation:

- reduce by surface area in phases;
- remove one product cluster at a time and keep the workspace boot graph compiling after each cut.

## Parallelization

Once the adapter strategy for `cli_agent` is agreed, the cleanup can be split cleanly:

1. **Terminal/CLI-agent track**
   - isolate `cli_agent` and slash-command dependencies;
   - split terminal-core exports from legacy terminal add-ons.

2. **Workspace/root-view track**
   - remove non-core panels, launch flows, right-side panes, and workspace actions.

3. **Product-surface deletion track**
   - delete code review, drive, workflow, notebook, pricing/billing/resource-center, and related settings/menu surfaces in logical groups.

4. **Dependency-pruning track**
   - remove unused crate dependencies and workspace members after code references are gone.

## Testing and validation

Validation should prove both that core workflows still work and that deleted surfaces are actually gone.

### Core workflow validation

- Build and launch the terminal-core app with `TERMINAL_CORE_MODE` enabled.
- Verify terminal creation, shell launch, pane splitting, tabs, command execution, history, completion, and settings/theme behavior still work.
- Verify `cli_agent` detection still works for supported agents and that rich input/session tracking still function.
- Verify terminal-adjacent file opening/editing still works for the retained editor/code path.

### Negative validation

- Confirm deleted surfaces no longer initialize from `app/src/lib.rs:1523-1579` and `app/src/workspace/mod.rs:101-132`.
- Confirm deleted commands, menus, settings pages, and modals are not merely hidden behind feature flags.
- Confirm `app/Cargo.toml` and workspace members no longer pull in deleted product dependencies.

### Incremental validation strategy

For each cleanup phase:

1. compile the app;
2. run the existing tests relevant to the touched area;
3. manually verify terminal creation and one `cli_agent` flow;
4. confirm no removed UI entrypoints still appear in menus, settings, or workspace actions.

## Follow-ups

- Add a sibling `PRODUCT.md` if we want a separate user-facing statement of what Miniwarp should and should not include.
- After the first major cleanup pass, update this spec so the “current state” section reflects the reduced architecture rather than the pre-cleanup repo.
