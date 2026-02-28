use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window};
use ui::{Color, KeystrokeRecordingState, Label, LabelCommon, LabelSize, h_flex, text_for_keystrokes};

use crate::{ItemHandle, StatusItemView};

/// Status bar item that shows the current keystroke sequence being recorded.
///
/// Displays "ctrl-h..." when recording a multi-key sequence.
/// Hidden when not recording.
pub struct KeystrokeRecordingIndicator {}

impl KeystrokeRecordingIndicator {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>) -> Self {
        Self {}
    }
}

impl Render for KeystrokeRecordingIndicator {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(state) = cx.try_global::<KeystrokeRecordingState>() else {
            return gpui::Empty.into_any_element();
        };

        if !state.is_recording {
            return gpui::Empty.into_any_element();
        }

        let display_text = if state.current_sequence.is_empty() {
            String::from("Recording...")
        } else {
            text_for_keystrokes(&state.current_sequence, cx)
        };

        h_flex()
            .gap_1()
            .child(
                Label::new(display_text)
                    .size(LabelSize::Small)
                    .color(Color::Accent),
            )
            .child(
                Label::new("...")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
            .into_any_element()
    }
}

impl StatusItemView for KeystrokeRecordingIndicator {
    fn set_active_pane_item(
        &mut self,
        _active_pane_item: Option<&dyn ItemHandle>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }
}
