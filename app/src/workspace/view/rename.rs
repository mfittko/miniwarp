use super::*;

impl Workspace {
    pub(super) fn tab_rename_editor_font_size(ctx: &AppContext, appearance: &Appearance) -> f32 {
        if FeatureFlag::VerticalTabs.is_enabled() && *TabSettings::as_ref(ctx).use_vertical_tabs {
            match *TabSettings::as_ref(ctx)
                .vertical_tabs_display_granularity
                .value()
            {
                VerticalTabsDisplayGranularity::Panes => 10.,
                VerticalTabsDisplayGranularity::Tabs => 12.,
            }
        } else {
            appearance.ui_font_size()
        }
    }

    pub(super) fn tab_rename_editor(ctx: &mut ViewContext<Self>) -> ViewHandle<EditorView> {
        let editor = {
            ctx.add_typed_action_view(|ctx| {
                let appearance = Appearance::as_ref(ctx);
                let options = SingleLineEditorOptions {
                    text: TextOptions::ui_text(
                        Some(Self::tab_rename_editor_font_size(ctx, appearance)),
                        appearance,
                    ),
                    ..Default::default()
                };
                EditorView::single_line(options, ctx)
            })
        };
        ctx.subscribe_to_view(&editor, move |me, _, event, ctx| {
            me.handle_tab_rename_editor_event(event, ctx);
        });
        editor
    }

    pub(super) fn pane_rename_editor(ctx: &mut ViewContext<Self>) -> ViewHandle<EditorView> {
        let editor = ctx.add_typed_action_view(|ctx| {
            let appearance = Appearance::as_ref(ctx);
            let options = SingleLineEditorOptions {
                text: TextOptions::ui_text(Some(12.), appearance),
                ..Default::default()
            };
            EditorView::single_line(options, ctx)
        });
        ctx.subscribe_to_view(&editor, move |me, _, event, ctx| {
            me.handle_pane_rename_editor_event(event, ctx);
        });
        editor
    }

    pub fn handle_tab_rename_editor_event(
        &mut self,
        event: &EditorEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        if self.current_workspace_state.is_tab_being_renamed() {
            match event {
                EditorEvent::Blurred | EditorEvent::Enter => {
                    self.finish_tab_rename(ctx);
                }
                EditorEvent::Escape => {
                    self.cancel_tab_rename(ctx);
                }
                _ => {}
            }
        }
    }

    pub fn handle_pane_rename_editor_event(
        &mut self,
        event: &EditorEvent,
        ctx: &mut ViewContext<Self>,
    ) {
        if self.current_workspace_state.is_any_pane_being_renamed() {
            match event {
                EditorEvent::Blurred | EditorEvent::Enter => {
                    self.finish_pane_rename(ctx);
                }
                EditorEvent::Escape => {
                    self.cancel_pane_rename(ctx);
                }
                _ => {}
            }
        }
    }

    pub(super) fn finish_tab_rename(&mut self, ctx: &mut ViewContext<Self>) {
        if let Some(tab_index) = self.current_workspace_state.tab_being_renamed() {
            self.current_workspace_state.clear_tab_being_renamed();
            let title = self.tab_rename_editor.as_ref(ctx).buffer_text(ctx);
            let tab = &self.tabs[tab_index];
            tab.pane_group.update(ctx, |view, ctx| {
                // Only update the title if it was actually changed. Otherwise, lets assume
                // user's intend was to cancel the operation.
                if view.display_title(ctx) != title {
                    send_telemetry_from_ctx!(
                        TelemetryEvent::TabRenamed(TabRenameEvent::CustomNameSet),
                        ctx
                    );
                    view.set_title(&title, ctx);
                }
            });
            self.clear_tab_name_editor(ctx);
            self.update_window_title(ctx);
            ctx.notify();
        }
    }

    pub(super) fn finish_pane_rename(&mut self, ctx: &mut ViewContext<Self>) {
        let Some(locator) = self.current_workspace_state.pane_being_renamed() else {
            return;
        };

        self.current_workspace_state.clear_pane_being_renamed();
        let title = self.pane_rename_editor.as_ref(ctx).buffer_text(ctx);
        self.set_custom_pane_name(locator, title, ctx);
        self.clear_pane_name_editor(ctx);
        self.focus_pane(locator, ctx);
        ctx.dispatch_global_action("workspace:save_app", ());
        ctx.notify();
    }

    pub(super) fn cancel_tab_rename(&mut self, ctx: &mut ViewContext<Self>) {
        if self.current_workspace_state.is_tab_being_renamed() {
            self.current_workspace_state.clear_tab_being_renamed();
            self.clear_tab_name_editor(ctx);
            self.focus_active_tab(ctx);
            ctx.notify();
        }
    }

    pub(super) fn cancel_pane_rename(&mut self, ctx: &mut ViewContext<Self>) {
        if let Some(locator) = self.current_workspace_state.pane_being_renamed() {
            self.current_workspace_state.clear_pane_being_renamed();
            self.clear_pane_name_editor(ctx);
            self.focus_pane(locator, ctx);
            ctx.notify();
        }
    }
}
