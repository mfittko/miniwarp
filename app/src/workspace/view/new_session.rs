use super::*;

impl Workspace {
    fn terminal_new_session_menu_item(shortcut_label: Option<String>) -> MenuItem<WorkspaceAction> {
        let mut item = MenuItemFields::new("Terminal")
            .with_on_select_action(WorkspaceAction::AddTerminalTab {
                hide_homepage: false,
            })
            .with_icon(icons::Icon::LayoutAlt01);
        if let Some(shortcut_label) = shortcut_label {
            item = item.with_key_shortcut_label(Some(shortcut_label));
        }
        item.into_item()
    }

    /// Clears the worktree sidecar state and hides the sidecar.
    pub(super) fn clear_worktree_sidecar_state(&mut self, ctx: &mut ViewContext<Self>) {
        self.show_new_session_sidecar = false;
        self.worktree_sidecar_active = false;
        self.worktree_sidecar_search_query.clear();
        self.worktree_sidecar_search_editor
            .update(ctx, |editor, ctx| {
                editor.clear_buffer(ctx);
            });
        self.new_session_sidecar_menu.update(ctx, |menu, view_ctx| {
            menu.clear_pinned_header_builder();
            menu.clear_pinned_footer_builder();
            menu.set_content_padding_overrides(None, None);
            menu.reset_selection(view_ctx);
        });
    }

    pub(super) fn close_new_session_dropdown_menu(&mut self, ctx: &mut ViewContext<Self>) {
        self.show_new_session_dropdown_menu = None;
        self.tab_config_action_sidecar_item = None;
        self.clear_worktree_sidecar_state(ctx);
        self.new_session_dropdown_menu.update(ctx, |menu, _| {
            menu.set_safe_zone_target(None);
            menu.set_submenu_being_shown_for_item_index(None);
        });
        ctx.notify();
    }

    pub(super) fn select_first_worktree_sidecar_repo(&mut self, ctx: &mut ViewContext<Self>) {
        self.new_session_sidecar_menu.update(ctx, |menu, view_ctx| {
            if menu.items_len() > 1 {
                menu.set_selected_by_index(1, view_ctx);
            } else {
                menu.reset_selection(view_ctx);
            }
        });
    }

    pub(super) fn reset_worktree_sidecar_repo_selection(&mut self, ctx: &mut ViewContext<Self>) {
        self.new_session_sidecar_menu.update(ctx, |menu, view_ctx| {
            menu.reset_selection(view_ctx);
        });
    }

    pub(super) fn navigate_worktree_sidecar_selection(
        &mut self,
        select_next: bool,
        ctx: &mut ViewContext<Self>,
    ) {
        self.new_session_sidecar_menu.update(ctx, |menu, view_ctx| {
            let items_len = menu.items_len();
            if items_len <= 1 {
                return;
            }

            match menu.selected_index() {
                Some(_) if select_next => menu.select_next(view_ctx),
                Some(_) => menu.select_previous(view_ctx),
                None if select_next => menu.set_selected_by_index(1, view_ctx),
                None => menu.set_selected_by_index(items_len.saturating_sub(1), view_ctx),
            }
        });
    }

    pub(super) fn confirm_worktree_sidecar_selection(&mut self, ctx: &mut ViewContext<Self>) {
        let selected_selection = self.new_session_sidecar_menu.update(ctx, |menu, view_ctx| {
            if menu.items_len() <= 1 {
                return None;
            }

            if menu.selected_index().is_none() {
                menu.set_selected_by_index(1, view_ctx);
            }

            menu.selected_item().and_then(|item| match item {
                MenuItem::Item(fields) => fields.on_select_action().cloned(),
                _ => None,
            })
        });

        if let Some(selection) = selected_selection {
            self.execute_new_session_sidecar_selection(selection, ctx);
            self.close_new_session_dropdown_menu(ctx);
        }
    }

    pub(super) fn sync_new_session_sidecar_selection_to_hover(
        &mut self,
        ctx: &mut ViewContext<Self>,
    ) {
        self.new_session_sidecar_menu.update(ctx, |menu, view_ctx| {
            let Some(hovered_index) = menu.hovered_index() else {
                return;
            };
            let hovered_item_has_action = menu
                .items()
                .get(hovered_index)
                .and_then(MenuItem::item_on_select_action)
                .is_some();

            if hovered_item_has_action && menu.selected_index() != Some(hovered_index) {
                menu.set_selected_by_index(hovered_index, view_ctx);
            }
        });
    }

    pub(super) fn build_worktree_sidecar_search_input(
        ctx: &mut ViewContext<Self>,
    ) -> ViewHandle<EditorView> {
        let editor = ctx.add_typed_action_view(|ctx| {
            let appearance = Appearance::as_ref(ctx);
            let mut editor = EditorView::single_line(
                SingleLineEditorOptions {
                    text: TextOptions::ui_text(Some(appearance.ui_font_size()), appearance),
                    select_all_on_focus: true,
                    clear_selections_on_blur: true,
                    propagate_and_no_op_vertical_navigation_keys:
                        PropagateAndNoOpNavigationKeys::Always,
                    ..Default::default()
                },
                ctx,
            );
            editor.set_placeholder_text("Search repos", ctx);
            editor
        });
        ctx.subscribe_to_view(&editor, |me, editor_view, event, ctx| match event {
            EditorEvent::Edited(_) => {
                me.worktree_sidecar_search_query = editor_view.as_ref(ctx).buffer_text(ctx);
                me.refresh_worktree_sidecar_if_active(ctx);
                ctx.notify();
            }
            EditorEvent::Escape => {
                me.close_new_session_dropdown_menu(ctx);
            }
            EditorEvent::Navigate(NavigationKey::Up) => {
                me.navigate_worktree_sidecar_selection(false, ctx);
            }
            EditorEvent::Navigate(NavigationKey::Down) => {
                me.navigate_worktree_sidecar_selection(true, ctx);
            }
            EditorEvent::Enter => {
                me.confirm_worktree_sidecar_selection(ctx);
            }
            _ => {}
        });
        editor
    }

    pub(super) fn unified_new_session_menu_items(
        &self,
        ctx: &mut ViewContext<Self>,
    ) -> Vec<MenuItem<WorkspaceAction>> {
        if is_terminal_core_mode() {
            let shortcut_label = keybinding_name_to_display_string(NEW_TAB_BINDING_NAME, ctx);
            let mut menu_items = vec![];

            #[cfg(target_os = "windows")]
            {
                menu_items.push(Self::terminal_new_session_menu_item(Some(
                    shortcut_label.clone(),
                )));

                #[cfg(feature = "local_tty")]
                if FeatureFlag::ShellSelector.is_enabled() {
                    AvailableShells::handle(ctx).read(ctx, |model, _| {
                        for shell in model.get_available_shells() {
                            let shell_name = model.display_name_for_shell(shell);
                            let icon = shell
                                .get_valid_shell_path_and_type()
                                .and_then(|shell_launch_data| {
                                    ShellIndicatorType::try_from(&shell_launch_data).ok()
                                })
                                .map(|shell_indicator_type| shell_indicator_type.to_icon())
                                .unwrap_or(icons::Icon::Terminal);
                            menu_items.push(
                                MenuItemFields::new(shell_name)
                                    .with_on_select_action(WorkspaceAction::AddTabWithShell {
                                        shell: shell.clone(),
                                        source: AddTabWithShellSource::ShellSelectorMenu,
                                    })
                                    .with_icon(icon)
                                    .into_item(),
                            );
                        }
                    });
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                menu_items.push(Self::terminal_new_session_menu_item(shortcut_label));
            }

            return menu_items;
        }

        let mut menu_items = vec![];

        let is_any_ai_enabled = AISettings::as_ref(ctx).is_any_ai_enabled(ctx);
        let ai_settings = AISettings::as_ref(ctx);
        let effective_default = ai_settings.default_session_mode(ctx);
        let default_tab_config_path = ai_settings.default_tab_config_path().to_string();
        let shortcut_label = keybinding_name_to_display_string(NEW_TAB_BINDING_NAME, ctx);

        if is_any_ai_enabled {
            let mut agent_item = MenuItemFields::new("Agent")
                .with_on_select_action(WorkspaceAction::AddAgentTab)
                .with_icon(icons::Icon::LayoutAlt01);
            if effective_default == DefaultSessionMode::Agent {
                agent_item = agent_item.with_key_shortcut_label(shortcut_label.clone());
            }
            menu_items.push(agent_item.into_item());
        }

        {
            #[cfg(target_os = "windows")]
            {
                let is_terminal_default = effective_default == DefaultSessionMode::Terminal;
                menu_items.push(Self::terminal_new_session_menu_item(
                    is_terminal_default.then(|| shortcut_label.clone()),
                ));

                #[cfg(feature = "local_tty")]
                if FeatureFlag::ShellSelector.is_enabled() {
                    AvailableShells::handle(ctx).read(ctx, |model, _| {
                        for shell in model.get_available_shells() {
                            let shell_name = model.display_name_for_shell(shell);
                            let icon = shell
                                .get_valid_shell_path_and_type()
                                .and_then(|shell_launch_data| {
                                    ShellIndicatorType::try_from(&shell_launch_data).ok()
                                })
                                .map(|shell_indicator_type| shell_indicator_type.to_icon())
                                .unwrap_or(icons::Icon::Terminal);
                            let item = MenuItemFields::new(shell_name)
                                .with_on_select_action(WorkspaceAction::AddTabWithShell {
                                    shell: shell.clone(),
                                    source: AddTabWithShellSource::ShellSelectorMenu,
                                })
                                .with_icon(icon);
                            menu_items.push(item.into_item());
                        }
                    });
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                menu_items.push(Self::terminal_new_session_menu_item(
                    if effective_default == DefaultSessionMode::Terminal {
                        shortcut_label.clone()
                    } else {
                        None
                    },
                ));
            }
        }

        if is_any_ai_enabled
            && FeatureFlag::AgentView.is_enabled()
            && FeatureFlag::CloudMode.is_enabled()
        {
            let mut cloud_item = MenuItemFields::new("Cloud Oz")
                .with_on_select_action(WorkspaceAction::AddAmbientAgentTab)
                .with_icon(icons::Icon::LayoutAlt01);
            if effective_default == DefaultSessionMode::CloudAgent {
                cloud_item = cloud_item.with_key_shortcut_label(shortcut_label.clone());
            }
            menu_items.push(cloud_item.into_item());
        }

        if FeatureFlag::LocalDockerSandbox.is_enabled() {
            let mut docker_item = MenuItemFields::new("Local Docker Sandbox")
                .with_on_select_action(WorkspaceAction::AddDockerSandboxTab)
                .with_icon(icons::Icon::Docker);
            if effective_default == DefaultSessionMode::DockerSandbox {
                docker_item = docker_item.with_key_shortcut_label(shortcut_label.clone());
            }
            menu_items.push(docker_item.into_item());
        }

        if FeatureFlag::TabConfigs.is_enabled() {
            let tab_configs = WarpConfig::as_ref(ctx).tab_configs().to_vec();

            let mut name_totals: HashMap<String, usize> = HashMap::new();
            for config in &tab_configs {
                *name_totals.entry(config.name.clone()).or_default() += 1;
            }
            let mut name_seen: HashMap<String, usize> = HashMap::new();

            for tab_config in tab_configs {
                let is_worktree = tab_config.is_worktree();
                let icon = if is_worktree {
                    icons::Icon::Dataflow02
                } else {
                    icons::Icon::LayoutAlt01
                };
                let is_default_config = effective_default == DefaultSessionMode::TabConfig
                    && tab_config
                        .source_path
                        .as_ref()
                        .is_some_and(|p| p.to_string_lossy() == default_tab_config_path);

                let display_name = if name_totals.get(&tab_config.name).copied().unwrap_or(0) > 1 {
                    let seen = name_seen.entry(tab_config.name.clone()).or_default();
                    *seen += 1;
                    if *seen == 1 {
                        tab_config.name.clone()
                    } else {
                        format!("{} ({})", tab_config.name, *seen - 1)
                    }
                } else {
                    tab_config.name.clone()
                };

                let mut item = MenuItemFields::new(display_name)
                    .with_on_select_action(WorkspaceAction::SelectTabConfig(tab_config))
                    .with_icon(icon);
                if is_default_config {
                    item = item.with_key_shortcut_label(shortcut_label.clone());
                }
                menu_items.push(item.into_item());
            }
        }

        if FeatureFlag::TabConfigs.is_enabled() {
            menu_items.push(MenuItem::Separator);
            menu_items.push(
                MenuItemFields::new_submenu("New worktree config")
                    .with_icon(icons::Icon::Dataflow02)
                    .into_item(),
            );

            menu_items.push(
                MenuItemFields::new("New tab config")
                    .with_on_select_action(WorkspaceAction::SelectNewSessionMenuItem(
                        NewSessionMenuItem::CreateNewTabConfig,
                    ))
                    .with_icon(icons::Icon::Plus)
                    .into_item(),
            );
        }

        menu_items
    }

    pub(super) fn open_tab_configs_menu(
        &mut self,
        position: Vector2F,
        is_vertical_tabs: bool,
        open_source: TabConfigsMenuOpenSource,
        ctx: &mut ViewContext<Self>,
    ) {
        let menu_items = self.unified_new_session_menu_items(ctx);
        ctx.update_view(&self.new_session_dropdown_menu, |context_menu, view_ctx| {
            if is_vertical_tabs {
                context_menu.set_width(268.);
            } else {
                context_menu.set_width(MENU_DEFAULT_WIDTH);
            }
            context_menu.set_items(menu_items, view_ctx);
            match open_source {
                TabConfigsMenuOpenSource::KeyboardShortcut => {
                    context_menu.set_selected_by_index(0, view_ctx);
                }
                TabConfigsMenuOpenSource::Pointer => {
                    context_menu.reset_selection(view_ctx);
                }
            }
        });
        self.show_new_session_dropdown_menu = Some(position);
        ctx.focus(&self.new_session_dropdown_menu);
        ctx.notify();
    }

    pub fn open_new_session_dropdown_menu(
        &mut self,
        position: Vector2F,
        ctx: &mut ViewContext<Self>,
    ) {
        self.open_tab_configs_menu(position, false, TabConfigsMenuOpenSource::Pointer, ctx);
    }

    pub(super) fn toggle_tab_configs_menu(&mut self, ctx: &mut ViewContext<Self>) {
        let use_vertical_tabs =
            FeatureFlag::VerticalTabs.is_enabled() && *TabSettings::as_ref(ctx).use_vertical_tabs;
        if self.show_new_session_dropdown_menu.is_some() {
            self.close_new_session_dropdown_menu(ctx);
            return;
        }

        if use_vertical_tabs {
            if !self.vertical_tabs_panel_open {
                self.vertical_tabs_panel_open = true;
                self.sync_window_button_visibility(ctx);
            }
            self.open_tab_configs_menu(
                Vector2F::zero(),
                true,
                TabConfigsMenuOpenSource::KeyboardShortcut,
                ctx,
            );
            return;
        }

        let position = ctx
            .element_position_by_id_at_last_frame(self.window_id, NEW_TAB_BUTTON_POSITION_ID)
            .map(|position| position.lower_left())
            .unwrap_or_else(Vector2F::zero);
        self.open_tab_configs_menu(
            position,
            false,
            TabConfigsMenuOpenSource::KeyboardShortcut,
            ctx,
        );
    }

    pub fn toggle_new_session_dropdown_menu(
        &mut self,
        position: Vector2F,
        is_vertical_tabs: bool,
        ctx: &mut ViewContext<Self>,
    ) {
        if self.show_new_session_dropdown_menu.is_some() {
            self.close_new_session_dropdown_menu(ctx);
            return;
        }

        self.open_tab_configs_menu(
            position,
            is_vertical_tabs,
            TabConfigsMenuOpenSource::Pointer,
            ctx,
        );
    }

    pub(super) fn handle_add_default_tab_action(&mut self, ctx: &mut ViewContext<Self>) {
        if is_terminal_core_mode() {
            self.add_terminal_tab(false, ctx);
            return;
        }
        let effective_mode = AISettings::as_ref(ctx).default_session_mode(ctx);
        match effective_mode {
            DefaultSessionMode::TabConfig => {
                let ai_settings = AISettings::as_ref(ctx);
                if let Some(config) = ai_settings.resolved_default_tab_config(ctx) {
                    self.open_tab_config(config, ctx);
                } else {
                    AISettings::handle(ctx).update(ctx, |settings, ctx| {
                        report_if_error!(settings
                            .default_session_mode_internal
                            .set_value(DefaultSessionMode::Terminal, ctx));
                        report_if_error!(settings
                            .default_tab_config_path
                            .set_value(String::new(), ctx));
                    });
                    self.add_terminal_tab(false, ctx);
                }
            }
            DefaultSessionMode::CloudAgent => {
                self.add_ambient_agent_tab(ctx);
            }
            DefaultSessionMode::DockerSandbox => {
                self.add_docker_sandbox_tab(ctx);
            }
            DefaultSessionMode::Terminal | DefaultSessionMode::Agent => {
                if FeatureFlag::WelcomeTab.is_enabled() {
                    self.add_welcome_tab(ctx);
                } else {
                    self.add_terminal_tab(false, ctx);
                }
            }
        }
    }

    pub(super) fn handle_add_ambient_agent_tab_action(&mut self, ctx: &mut ViewContext<Self>) {
        if is_terminal_core_mode() {
            self.add_terminal_tab(false, ctx);
        } else {
            self.add_ambient_agent_tab(ctx);
        }
    }

    pub(super) fn handle_add_agent_tab_action(&mut self, ctx: &mut ViewContext<Self>) {
        if is_terminal_core_mode() {
            self.add_terminal_tab(false, ctx);
        } else {
            self.add_terminal_tab_with_new_agent_view(ctx);
        }
    }

    pub(super) fn handle_add_docker_sandbox_tab_action(&mut self, ctx: &mut ViewContext<Self>) {
        if is_terminal_core_mode() {
            self.add_terminal_tab(false, ctx);
        } else {
            self.add_docker_sandbox_tab(ctx);
        }
    }

    pub(super) fn handle_start_agent_onboarding_tutorial_action(
        &mut self,
        tutorial: &OnboardingTutorial,
        ctx: &mut ViewContext<Self>,
    ) {
        if !is_terminal_core_mode() {
            self.start_agent_onboarding_tutorial(tutorial.clone(), ctx)
        }
    }
}
