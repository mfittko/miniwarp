use super::*;

impl Workspace {
    pub(super) fn open_left_panel(&mut self, ctx: &mut ViewContext<Self>) {
        if is_terminal_core_mode() {
            return;
        }
        self.left_panel_open = true;

        let active_pane_group = self.active_tab_pane_group().clone();
        active_pane_group.update(ctx, |pane_group, ctx| {
            pane_group.set_left_panel_open(true, ctx);
        });

        ctx.notify();
    }

    /// Auto-opens the conversation list on first app start.
    /// Once we've done this once, we persist a preference so subsequent restarts
    /// will respect the user's visibility preference (restored from workspace state).
    pub(super) fn maybe_auto_open_conversation_list(&mut self, ctx: &mut ViewContext<Self>) {
        if is_terminal_core_mode() {
            return;
        }
        if !FeatureFlag::AgentViewConversationListView.is_enabled()
            || !AISettings::as_ref(ctx).is_any_ai_enabled(ctx)
        {
            return;
        }

        let has_auto_opened = *AISettings::as_ref(ctx).has_auto_opened_conversation_list;
        if has_auto_opened {
            return;
        }

        let has_terminal = self
            .active_tab_pane_group()
            .as_ref(ctx)
            .has_terminal_panes();
        if !has_terminal {
            return;
        }

        if !self.active_tab_pane_group().as_ref(ctx).left_panel_open {
            self.open_left_panel(ctx);
        }
        self.left_panel_view.update(ctx, |lp, ctx| {
            lp.restore_active_view_from_snapshot(ToolPanelView::ConversationListView, ctx);
        });

        AISettings::handle(ctx).update(ctx, |settings, ctx| {
            report_if_error!(settings
                .has_auto_opened_conversation_list
                .set_value(true, ctx));
        });
    }

    pub(super) fn close_left_panel(&mut self, ctx: &mut ViewContext<Self>) {
        self.left_panel_open = false;

        let active_pane_group = self.active_tab_pane_group().clone();
        active_pane_group.update(ctx, |pane_group, ctx| {
            pane_group.set_left_panel_open(false, ctx);
        });

        ctx.notify();
    }

    pub(super) fn toggle_vertical_tabs_panel(&mut self, ctx: &mut ViewContext<Self>) {
        self.vertical_tabs_panel_open = !self.vertical_tabs_panel_open;
        if !self.vertical_tabs_panel_open {
            self.close_vertical_tabs_settings_popup();
            self.vertical_tabs_panel.clear_detail_sidecar();
        }
        self.sync_window_button_visibility(ctx);
        ctx.notify();
    }

    pub(super) fn close_vertical_tabs_settings_popup(&mut self) {
        self.vertical_tabs_panel.show_settings_popup = false;
    }

    /// Sets the visibility state of the agent management view
    /// and updates the AgentConversationsModel to reflect the new state.
    pub(super) fn set_is_agent_management_view_open(
        &mut self,
        is_open: bool,
        ctx: &mut ViewContext<Self>,
    ) {
        let was_open = self.current_workspace_state.is_agent_management_view_open;
        if was_open == is_open {
            return;
        }
        self.current_workspace_state.is_agent_management_view_open = is_open;
        let window_id = self.window_id;
        let view_id = self.agent_management_view.id();
        AgentConversationsModel::handle(ctx).update(ctx, |model, ctx| {
            if is_open {
                model.register_view_open(window_id, view_id, ctx);
            } else {
                model.register_view_closed(window_id, view_id, ctx);
            }
        });

        self.left_panel_view.update(ctx, |panel, ctx| {
            panel.set_agent_management_view_open(is_open, ctx);
        });
        self.right_panel_view.update(ctx, |panel, ctx| {
            panel.set_agent_management_view_open(is_open, ctx);
        });
    }

    pub(super) fn toggle_left_panel(&mut self, ctx: &mut ViewContext<Self>) {
        if is_terminal_core_mode() {
            return;
        }
        let active_pane_group = self.active_tab_pane_group().clone();

        let was_open = active_pane_group.read(ctx, |pane_group, _| pane_group.left_panel_open);
        let new_state = !was_open;

        if new_state {
            self.open_left_panel(ctx);
        } else {
            self.close_left_panel(ctx);
        }

        if new_state {
            let window_id = ctx.window_id();
            let resizable_data = ResizableData::handle(ctx);
            if let Some(handle) = resizable_data
                .as_ref(ctx)
                .get_handle(window_id, ModalType::LeftPanelWidth)
            {
                if let Ok(mut state) = handle.lock() {
                    let current_width = state.size();

                    if current_width == DEFAULT_LEFT_PANEL_WIDTH {
                        let has_horizontal_split = active_pane_group
                            .read(ctx, |pane_group, _| pane_group.has_horizontal_split());
                        let (left_width, _right_width) =
                            compute_default_panel_widths(ctx, window_id, has_horizontal_split);
                        state.set_size(left_width);
                    }
                }
            }

            let file_tree_active = self
                .left_panel_view
                .read(ctx, |lp, _| lp.is_file_tree_active());
            if file_tree_active {
                self.left_panel_view.update(ctx, |left_panel, ctx| {
                    left_panel.auto_expand_active_file_tree_to_most_recent_directory(ctx);
                });
            }
        }

        if !new_state {
            self.focus_active_tab(ctx);
        }

        ctx.notify();
    }

    #[cfg(feature = "local_fs")]
    pub(super) fn setup_code_review_panel(
        &mut self,
        context: Option<&CodeReviewPaneContext>,
        ctx: &mut ViewContext<Self>,
    ) {
        if !*TabSettings::as_ref(ctx).show_code_review_button {
            return;
        }

        let context_data: Option<(
            Option<PathBuf>,
            ModelHandle<DiffStateModel>,
            WeakViewHandle<TerminalView>,
        )> = if let Some(context) = context {
            Some((
                context.repo_path.clone(),
                context.diff_state_model.clone(),
                context.terminal_view.clone(),
            ))
        } else {
            let active_pane_group = self.active_tab_pane_group().clone();
            let read_result = active_pane_group.read(ctx, |pane_group, ctx| {
                pane_group.active_session_view(ctx).map(|terminal_view| {
                    let repo_path = terminal_view.as_ref(ctx).current_repo_path().cloned();
                    (repo_path, terminal_view.downgrade())
                })
            });
            read_result.and_then(
                |(repo_path, terminal_view): (Option<PathBuf>, WeakViewHandle<TerminalView>)| {
                    let diff_state_model = repo_path.as_ref().and_then(|rp: &PathBuf| {
                        self.working_directories_model.update(ctx, |model, ctx| {
                            model.get_or_create_diff_state_model(rp.clone(), ctx)
                        })
                    })?;
                    Some((repo_path, diff_state_model, terminal_view))
                },
            )
        };

        if let Some((repo, diff_state_model, terminal_view)) = context_data {
            self.right_panel_view.update(ctx, |right_pane_view, ctx| {
                right_pane_view.open_code_review(
                    repo.clone(),
                    diff_state_model,
                    terminal_view,
                    ctx,
                );
            });
        } else {
            self.right_panel_view.update(ctx, |right_panel_view, ctx| {
                right_panel_view.close_code_review(ctx);
            })
        }
    }

    pub(super) fn open_code_review_panel_from_arg(
        &mut self,
        panel_context: &CodeReviewPanelArg,
        pane_group: ViewHandle<PaneGroup>,
        ctx: &mut ViewContext<Self>,
    ) {
        let panel_already_showing_repo = pane_group.as_ref(ctx).right_panel_open
            && panel_context
                .repo_path
                .as_ref()
                .is_some_and(|target_repo_path| {
                    self.right_panel_view.as_ref(ctx).selected_repo_path() == Some(target_repo_path)
                });
        if panel_already_showing_repo {
            return;
        }

        let repo_path = panel_context.repo_path.clone();
        let diff_state_model = repo_path.as_ref().and_then(|rp| {
            self.working_directories_model.update(ctx, |model, ctx| {
                model.get_or_create_diff_state_model(rp.clone(), ctx)
            })
        });
        let Some(diff_state_model) = diff_state_model else {
            return;
        };
        let context = CodeReviewPaneContext {
            repo_path,
            diff_state_model,
            terminal_view: panel_context.terminal_view.clone(),
        };

        self.open_right_panel(
            &context,
            &pane_group,
            panel_context.entrypoint,
            panel_context.cli_agent,
            ctx,
        );

        let active_conversation_id = panel_context
            .terminal_view
            .upgrade(ctx)
            .and_then(|tv| BlocklistAIHistoryModel::as_ref(ctx).active_conversation_id(tv.id()));

        if let Some(conversation_id) = active_conversation_id {
            BlocklistAIHistoryModel::handle(ctx).update(ctx, |history_model, _| {
                history_model.set_has_code_review_opened_to_true(conversation_id);
            });
        }
    }

    pub(super) fn update_right_panel_open_state(
        &mut self,
        #[cfg_attr(target_family = "wasm", allow(unused_variables))]
        panel_update_params: RightPanelUpdateParams,
        ctx: &mut ViewContext<Self>,
    ) {
        let should_open = panel_update_params.target_open_state;
        let should_close = !panel_update_params.target_open_state;

        let new_is_maximized = panel_update_params.pane_group.update(ctx, |pane_group, _| {
            pane_group.right_panel_open = should_open;
            pane_group.is_right_panel_maximized
        });

        self.right_panel_view.update(ctx, |view, ctx| {
            view.set_maximized(new_is_maximized, ctx);
            if should_close {
                view.close_code_review(ctx);
            }
        });

        if should_open {
            #[cfg(feature = "local_fs")]
            {
                let window_id = ctx.window_id();
                let resizable_data = ResizableData::handle(ctx);
                if let Some(handle) = resizable_data
                    .as_ref(ctx)
                    .get_handle(window_id, ModalType::RightPanelWidth)
                {
                    if let Ok(mut state) = handle.lock() {
                        let current_width = state.size();

                        if current_width == DEFAULT_RIGHT_PANEL_WIDTH {
                            let has_horizontal_split = panel_update_params
                                .pane_group
                                .read(ctx, |pane_group, _| pane_group.has_horizontal_split());
                            let (_left_width, right_width) =
                                compute_default_panel_widths(ctx, window_id, has_horizontal_split);
                            state.set_size(right_width);
                        }
                    }
                }
                send_telemetry_from_ctx!(
                    CodeReviewTelemetryEvent::PaneOpened {
                        entrypoint: panel_update_params.entrypoint.unwrap_or_default(),
                        is_code_mode_v2: true,
                        cli_agent: panel_update_params.cli_agent.map(Into::into),
                    },
                    ctx
                );
                self.setup_code_review_panel(panel_update_params.review_pane_context, ctx);
            }
        } else {
            self.focus_active_tab(ctx);
        }

        ctx.notify();
    }

    pub(super) fn toggle_right_panel(
        &mut self,
        pane_group_handle: &ViewHandle<PaneGroup>,
        ctx: &mut ViewContext<Self>,
    ) {
        if is_terminal_core_mode() {
            return;
        }
        let target_open_state =
            pane_group_handle.read(ctx, |pane_group, _| !pane_group.right_panel_open);

        let read_result = pane_group_handle.read(ctx, |pane_group, ctx| {
            pane_group.active_session_view(ctx).map(|terminal_view| {
                let repo_path = terminal_view.as_ref(ctx).current_repo_path().cloned();
                (repo_path, terminal_view.downgrade())
            })
        });
        let context = read_result.and_then(
            |(repo_path, terminal_view): (Option<PathBuf>, WeakViewHandle<TerminalView>)| {
                let diff_state_model = repo_path.as_ref().and_then(|rp: &PathBuf| {
                    self.working_directories_model.update(ctx, |model, ctx| {
                        model.get_or_create_diff_state_model(rp.clone(), ctx)
                    })
                })?;
                Some(CodeReviewPaneContext {
                    repo_path,
                    diff_state_model,
                    terminal_view,
                })
            },
        );

        self.update_right_panel_open_state(
            RightPanelUpdateParams {
                pane_group: pane_group_handle,
                target_open_state,
                entrypoint: Some(CodeReviewPaneEntrypoint::RightPanel),
                cli_agent: None,
                review_pane_context: context.as_ref(),
            },
            ctx,
        );
    }

    pub fn close_right_panel(
        &mut self,
        pane_group_handle: &ViewHandle<PaneGroup>,
        ctx: &mut ViewContext<Self>,
    ) {
        self.update_right_panel_open_state(
            RightPanelUpdateParams {
                pane_group: pane_group_handle,
                target_open_state: false,
                entrypoint: None,
                cli_agent: None,
                review_pane_context: None,
            },
            ctx,
        );
    }

    #[cfg_attr(not(feature = "local_fs"), allow(dead_code))]
    pub(super) fn toggle_right_panel_maximized(&mut self, ctx: &mut ViewContext<Self>) {
        let pane_group = self.active_tab_pane_group().clone();
        let is_maximized = pane_group.update(ctx, |pane_group, _| {
            pane_group.is_right_panel_maximized = !pane_group.is_right_panel_maximized;
            pane_group.is_right_panel_maximized
        });

        self.right_panel_view.update(ctx, |view, ctx| {
            view.set_maximized(is_maximized, ctx);
            if is_maximized {
                view.focus_active_code_review_view(ctx);
            }
        });
        if !is_maximized {
            self.focus_active_tab(ctx);
        }
        ctx.notify();
    }

    pub(super) fn open_left_panel_view(
        &mut self,
        action: &LeftPanelAction,
        ctx: &mut ViewContext<Self>,
    ) {
        if is_terminal_core_mode() {
            return;
        }
        if !self.active_tab_pane_group().as_ref(ctx).left_panel_open {
            self.toggle_left_panel(ctx);
        }

        if self.active_tab_pane_group().as_ref(ctx).left_panel_open {
            self.left_panel_view.update(ctx, |left_panel, ctx| {
                left_panel.handle_action_with_force_open(action, false, ctx);
                left_panel.focus_active_view_on_entry(ctx);
            });
        }
    }

    pub(super) fn toggle_left_panel_view(
        &mut self,
        action: &LeftPanelAction,
        is_showing_target_view: bool,
        ctx: &mut ViewContext<Self>,
    ) {
        if is_terminal_core_mode() {
            return;
        }
        let is_left_panel_open = self.active_tab_pane_group().as_ref(ctx).left_panel_open;

        if is_left_panel_open && is_showing_target_view {
            self.toggle_left_panel(ctx);
        } else {
            self.open_left_panel_view(action, ctx);
        }
    }

    pub(super) fn compute_left_panel_views(ctx: &AppContext) -> Vec<ToolPanelView> {
        if is_terminal_core_mode() {
            return vec![];
        }
        let mut views = vec![];
        if FeatureFlag::AgentViewConversationListView.is_enabled()
            && AISettings::as_ref(ctx).is_any_ai_enabled(ctx)
            && *AISettings::as_ref(ctx).show_conversation_history
        {
            views.push(ToolPanelView::ConversationListView);
        }

        if cfg!(feature = "local_fs") && *CodeSettings::as_ref(ctx).show_project_explorer.value() {
            views.push(ToolPanelView::ProjectExplorer);
        }
        if cfg!(feature = "local_fs")
            && FeatureFlag::GlobalSearch.is_enabled()
            && *CodeSettings::as_ref(ctx).show_global_search.value()
        {
            views.push(ToolPanelView::GlobalSearch {
                entry_focus: GlobalSearchEntryFocus::Results,
            });
        }
        if WarpDriveSettings::is_warp_drive_enabled(ctx) {
            views.push(ToolPanelView::WarpDrive);
        }
        views
    }

    pub(super) fn update_left_panel_available_views(&mut self, ctx: &mut ViewContext<Self>) {
        let views = Self::compute_left_panel_views(ctx);
        self.left_panel_views = views.clone();
        self.left_panel_view.update(ctx, |left_panel, ctx| {
            left_panel.update_available_views(views, ctx);
        });
    }
}
