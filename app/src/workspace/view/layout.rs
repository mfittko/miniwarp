use super::*;

impl Workspace {
    /// Offset positioning for agent toasts.
    /// TODO: update positioning based on input mode.
    pub(super) fn agent_toast_positioning(&self) -> OffsetPositioning {
        OffsetPositioning::offset_from_save_position_element(
            TAB_CONTENT_POSITION_ID,
            vec2f(0., 16.),
            PositionedElementOffsetBounds::WindowByPosition,
            PositionedElementAnchor::TopRight,
            ChildAnchor::TopRight,
        )
    }

    /// Offset positioning for global toasts.
    // TODO: update positioning based on input mode.
    pub(super) fn global_toast_positioning(&self) -> OffsetPositioning {
        OffsetPositioning::offset_from_save_position_element(
            TAB_CONTENT_POSITION_ID,
            vec2f(0., 16.),
            PositionedElementOffsetBounds::WindowByPosition,
            PositionedElementAnchor::TopMiddle,
            ChildAnchor::TopMiddle,
        )
    }

    /// Offset positioning for the update toast.
    pub(super) fn update_toast_positioning(
        &self,
        input_position_id: String,
        app: &AppContext,
    ) -> OffsetPositioning {
        let input_mode = InputModeSettings::as_ref(app).input_mode.value();

        match input_mode {
            InputMode::PinnedToBottom => OffsetPositioning::offset_from_save_position_element(
                input_position_id,
                vec2f(-16., -16.),
                PositionedElementOffsetBounds::WindowByPosition,
                PositionedElementAnchor::TopRight,
                ChildAnchor::BottomRight,
            ),
            InputMode::PinnedToTop => OffsetPositioning::offset_from_save_position_element(
                input_position_id,
                vec2f(-16., 16.),
                PositionedElementOffsetBounds::WindowByPosition,
                PositionedElementAnchor::BottomRight,
                ChildAnchor::TopRight,
            ),
            InputMode::Waterfall => OffsetPositioning::offset_from_parent(
                vec2f(-16., -16.),
                ParentOffsetBounds::WindowByPosition,
                ParentAnchor::BottomRight,
                ChildAnchor::BottomRight,
            ),
        }
    }
}

pub(super) fn compute_default_panel_widths(
    app: &AppContext,
    window_id: WindowId,
    has_horizontal_split: bool,
) -> (f32, f32) {
    if let Some(bounds) = app.window_bounds(&window_id) {
        let window_width = bounds.width();
        let left_ratio = 0.15;
        let right_ratio = if has_horizontal_split { 0.3 } else { 0.5 };
        let left = window_width * left_ratio;
        let right = window_width * right_ratio;
        (left, right)
    } else {
        (DEFAULT_LEFT_PANEL_WIDTH, DEFAULT_RIGHT_PANEL_WIDTH)
    }
}
