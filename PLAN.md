# Miniwarp Reduction Plan

This plan is the execution checklist for reducing Warp into Miniwarp: a terminal-first app with only the minimum retained terminal-adjacent functionality.

## Phase 0 — Lock the target

- [ ] Approve the product boundary in `specs/terminal-core-simplification/PRODUCT.md`
- [ ] Approve the technical boundary in `specs/terminal-core-simplification/TECH.md`
- [ ] Treat `ARCHITECTURE.md` as the target-state reference for what remains
- [ ] Treat this file as the delivery checklist and keep it updated as work lands

## Phase 1 — Make terminal-core a real boot boundary

- [ ] Audit `/home/runner/work/miniwarp/miniwarp/app/src/lib.rs` boot/init order and classify every initialization point as **keep**, **gate temporarily**, or **delete**
- [ ] Gate or remove all obvious non-core initializers that still run unconditionally in terminal-core mode
- [ ] Reduce `TERMINAL_CORE_DISABLED_FEATURE_FLAGS` to a temporary migration aid rather than the main boundary
- [ ] Ensure terminal startup does not require non-core singletons, actions, or registrations
- [ ] Confirm the app still boots cleanly after the boot graph is reduced

## Phase 2 — Preserve `cli_agent` by isolating it

- [ ] Inventory all `cli_agent` dependencies on AI, code review, workflows, cloud objects, and workspace-only types
- [ ] Introduce terminal-core-owned adapter types for:
  - [ ] session status
  - [ ] rich-input mode/config
  - [ ] optional review/comment metadata only if still required
  - [ ] workspace hooks needed by terminal-hosted flows
- [ ] Refactor `/home/runner/work/miniwarp/miniwarp/app/src/terminal/cli_agent.rs` to depend on those adapters instead of broad product modules
- [ ] Refactor `/home/runner/work/miniwarp/miniwarp/app/src/terminal/cli_agent_sessions/mod.rs` to remove direct dependence on non-core conversation/input types
- [ ] Refactor `/home/runner/work/miniwarp/miniwarp/app/src/terminal/input/slash_commands/mod.rs` so terminal input no longer pulls in removed product surfaces
- [ ] Verify `cli_agent` detection, session tracking, and rich input still work after each cut

## Phase 3 — Trim terminal module to the true terminal surface

- [ ] Split retained terminal-core exports from legacy terminal add-ons in `/home/runner/work/miniwarp/miniwarp/app/src/terminal/mod.rs`
- [ ] Remove share/session/social surfaces that are not part of Miniwarp’s target product
- [ ] Remove buy-credits or similar product surfaces embedded in the terminal module
- [ ] Keep only terminal rendering, PTY/session management, history, shell launch, settings, SSH flows that are still in scope, and `cli_agent`
- [ ] Verify terminal initialization registers only the retained terminal-core subset

## Phase 4 — Simplify workspace to terminal orchestration

- [ ] Audit `/home/runner/work/miniwarp/miniwarp/app/src/workspace/mod.rs` and related workspace/root-view modules for product-specific imports, actions, and bindings
- [ ] Remove launch modals, right-panel surfaces, onboarding flows, and global product panes that are outside Miniwarp’s scope
- [ ] Remove notebook-, workflow-, and code-review-specific workspace actions and toolbar items
- [ ] Keep only the workspace pieces needed for terminal tabs, panes, lightweight file access, settings access, and essential toasts/modals
- [ ] Reconcile menu items, command-palette actions, and keybindings with the reduced workspace
- [ ] Verify the workspace can still host the retained terminal workflows without dead controls

## Phase 5 — Delete obvious product clusters

- [ ] Remove notebooks and their entry points
- [ ] Remove workflows and workflow-only UI/menus/settings
- [ ] Remove code review panes, comments UI, and review-only commands
- [ ] Remove drive/cloud-object-first UI that has no terminal-first justification
- [ ] Remove pricing, billing, credits, rewards, referrals, and upsell surfaces
- [ ] Remove resource-center/product-tour/onboarding surfaces that are not essential to using Miniwarp
- [ ] Remove ambient/persistent AI side panels and broader agent workspace surfaces outside retained explicit assistance
- [ ] After each cluster deletion, remove associated settings, menu items, telemetry hooks, and feature-flag plumbing

## Phase 6 — Prune dependencies and workspace members

- [ ] Review `/home/runner/work/miniwarp/miniwarp/app/Cargo.toml` and mark every dependency as **retain**, **candidate**, or **remove**
- [ ] Remove app dependencies that only support deleted surfaces
- [ ] Remove or stop building workspace crates that only serve deleted products
- [ ] Re-run compiler-driven cleanup until no deleted surfaces are referenced anywhere
- [ ] Prefer deleting dead modules/crates over carrying permanent feature-flagged stubs

## Phase 7 — Align product surface and docs

- [ ] Update README and any user-facing docs so Miniwarp is described only in terms of the retained product
- [ ] Remove outdated references to deleted product areas from developer docs where needed
- [ ] Refresh `ARCHITECTURE.md` to match the post-cleanup implementation state
- [ ] Refresh the specs so “current state” no longer describes removed systems

## Validation checklist

- [ ] Core terminal app builds
- [ ] Existing terminal-relevant tests pass for touched areas
- [ ] Terminal creation works
- [ ] Shell launch works
- [ ] Tabs and pane splitting work
- [ ] History and completion flows still work
- [ ] Theme/settings flows still work
- [ ] Retained terminal-adjacent file-open/edit flows still work
- [ ] `cli_agent` detection still works
- [ ] `cli_agent` session tracking still works
- [ ] Retained explicit `#`-triggered assistance still works, if kept
- [ ] Deleted surfaces do not appear in menus, settings, panels, or startup
- [ ] Removed dependencies are actually gone from the reachable build graph

## Suggested work order

1. Boot boundary
2. `cli_agent` isolation
3. Terminal module cleanup
4. Workspace/root-view cleanup
5. Product-cluster deletion
6. Dependency/crate pruning
7. Docs and naming cleanup
