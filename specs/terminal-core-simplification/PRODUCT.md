# Miniwarp Terminal-Core Reduction — Product Spec

## Summary

Miniwarp should become a focused terminal product: fast terminal sessions, tabs, panes, history, completions, shell integration, theme/settings support, lightweight file-opening/editing for terminal workflows, explicit `#`-triggered command assistance, and preserved `cli_agent` interoperability.

Everything else should be treated as removable unless it has a direct, terminal-first reason to stay.

## Problem

This fork still contains most of the broader Warp product surface even though its intended direction is much narrower. The current codebase still initializes and depends on AI chat surfaces, collaboration flows, notebooks, workflows, code review, pricing/billing UI, onboarding, and other cloud-heavy features that are outside Miniwarp’s goal.

That mismatch creates three problems:

1. the product boundary is unclear;
2. the codebase remains expensive to understand and change;
3. the shipped experience still carries complexity that Miniwarp does not want.

## Goals

- Define a clear product boundary for Miniwarp.
- Preserve excellent local terminal workflows: sessions, tabs, panes, shell launch, completions, history, navigation, themes, and settings.
- Keep minimal file-opening/editing support when it directly supports terminal use.
- Keep explicit `#`-triggered command assistance and `cli_agent` interoperability.
- Remove user-facing surfaces whose primary value comes from cloud sync, collaboration, shared sessions, notebooks, workflows, code review, billing/pricing, or upsell/product-tour flows.
- Make the reduced product understandable enough that future cleanup can be executed in phases.

## Non-goals

- Preserving feature parity with Warp OSS.
- Keeping dormant product areas hidden behind flags forever.
- Designing a new cloud product, chat application, or collaboration experience.
- Replacing `cli_agent` with Warp-hosted agent panes.
- Reworking core terminal UX beyond what is needed to simplify the product boundary.

## Figma

Figma: none provided. This is primarily a product-scope reduction and architecture simplification effort.

## Target user experience

Miniwarp should feel like a polished modern terminal, not a multi-product workspace shell.

### Included experience

1. **Terminal first.** Users can create terminal tabs and panes, run commands, inspect scrollback/history, use completions, and customize themes/settings.
2. **Shell integration.** Shell launch, session restoration where already supported, PTY-backed command execution, and terminal-specific keybindings remain first-class.
3. **Terminal-adjacent editing.** Opening and lightly editing files remains available when it directly supports terminal workflows.
4. **Explicit assistance only.** AI-adjacent help is limited to explicit `#`-triggered command assistance and retained `cli_agent` interoperability. There are no persistent AI side panels or product-wide agent surfaces.
5. **CLI-agent support.** When supported CLI agents are active, Miniwarp can still detect them, track their sessions, support rich input where applicable, and emit the necessary telemetry/hooks for that workflow.

### Excluded experience

Miniwarp should not expose or promote:

- notebooks;
- workflows as a standalone product;
- code review panes and saved-review flows;
- shared-session and collaboration/social surfaces;
- drive/cloud-object-first experiences;
- pricing, billing, credits, rewards, referrals, or upsell surfaces;
- resource-center/product-tour/onboarding surfaces that exist mainly for the broader Warp product;
- right-side AI/chat panels and ambient agent workspaces.

## Invariants

- Miniwarp must still launch as a usable terminal application.
- Terminal boot must not depend on deleted non-core product surfaces.
- `cli_agent` support must keep working throughout the reduction.
- Removed surfaces should be deleted or fully isolated, not merely hidden in menus.
- Any retained non-terminal subsystem needs a specific terminal-first justification.

## Edge cases

1. **Terminal-adjacent code stays only if justified.** Editor/code support may remain, but only where it clearly improves terminal use.
2. **AI-adjacent ambiguity.** Explicit `#`-triggered assistance and `cli_agent` interoperability may remain; always-on AI panes, workflow generation, and chat-centric product surfaces should not.
3. **Settings cleanup.** Settings for removed products must also disappear; Miniwarp should not expose dead or misleading configuration.
4. **Menu cleanup.** Commands, keyboard shortcuts, and menu entries for removed surfaces must be removed, not just no-op.
5. **Incremental delivery.** During transition, temporary gating is acceptable only when it clearly leads to deletion and a smaller reachable graph.

## Success criteria

1. Miniwarp boots into a terminal-first workspace without initializing the removed product clusters.
2. A user can still create tabs/panes, launch shells, run commands, use history/completions, and change terminal/theme settings.
3. Opening a file from terminal-adjacent flows still works where intentionally retained.
4. `cli_agent` detection, session tracking, and rich-input-related workflows still function.
5. Explicit `#`-triggered command assistance remains available if kept in scope.
6. Notebooks, workflows, code review, shared sessions, pricing/billing, resource-center upsell, and similar non-core surfaces are absent from the product experience.
7. The app’s retained architecture is understandable as a terminal-centered system rather than a feature-flagged superset product.

## Validation

- Verify the app still supports core terminal workflows end to end.
- Verify retained `cli_agent` flows still work.
- Verify removed surfaces no longer appear in startup, menus, settings, panels, or terminal add-ons.
- Verify the app’s dependency graph and initialization flow move toward the documented target architecture.
