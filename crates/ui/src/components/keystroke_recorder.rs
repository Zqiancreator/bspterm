use std::rc::Rc;

use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, Global, IntoElement, KeyDownEvent,
    Keystroke, ParentElement, Render, SharedString, Styled, Window,
};
use i18n::t;

use crate::{Color, Label, LabelSize, h_flex, prelude::*, v_flex};

/// Global state for keystroke recording, used to display recording status in the status bar.
#[derive(Default)]
pub struct KeystrokeRecordingState {
    pub current_sequence: Vec<Keystroke>,
    pub is_recording: bool,
}

impl Global for KeystrokeRecordingState {}

impl KeystrokeRecordingState {
    pub fn init(cx: &mut App) {
        cx.set_global(Self::default());
    }

    pub fn get(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn update(cx: &mut App, is_recording: bool, keystrokes: Vec<Keystroke>) {
        if cx.try_global::<Self>().is_some() {
            let state = cx.global_mut::<Self>();
            state.is_recording = is_recording;
            state.current_sequence = keystrokes;
        }
    }
}

/// Events emitted by the KeystrokeRecorder.
#[derive(Clone, Debug)]
pub enum KeystrokeRecorderEvent {
    /// Recording started.
    RecordingStarted,
    /// Recording was cancelled (Esc or ctrl-g).
    RecordingCancelled,
    /// Recording completed successfully with the given keybinding string.
    RecordingCompleted(String),
}

/// A component for interactively recording keyboard shortcuts.
///
/// Features:
/// - Click to start recording
/// - Press keys to build a multi-key sequence (e.g., "ctrl-h h")
/// - Press Enter to confirm the sequence
/// - Press Esc or ctrl-g to cancel
/// - No timeout between keystrokes
pub struct KeystrokeRecorder {
    focus_handle: FocusHandle,
    keystrokes: Vec<Keystroke>,
    is_recording: bool,
    value: Option<String>,
    on_change: Option<Rc<dyn Fn(String, &mut Window, &mut App)>>,
}

impl EventEmitter<KeystrokeRecorderEvent> for KeystrokeRecorder {}

impl KeystrokeRecorder {
    pub fn new(value: Option<String>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        cx.on_focus(&focus_handle, window, Self::on_focus).detach();

        cx.on_blur(&focus_handle, window, Self::on_blur).detach();

        Self {
            focus_handle,
            keystrokes: Vec::new(),
            is_recording: false,
            value,
            on_change: None,
        }
    }

    fn on_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.start_recording(window, cx);
    }

    fn on_blur(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_recording {
            self.cancel_recording(window, cx);
        }
    }

    /// Set a callback to be called when the recorded keybinding changes.
    pub fn set_on_change(&mut self, callback: impl Fn(String, &mut Window, &mut App) + 'static) {
        self.on_change = Some(Rc::new(callback));
    }

    /// Get the current recorded keybinding as a string.
    pub fn keybinding_string(&self) -> String {
        self.keystrokes
            .iter()
            .map(|k| k.unparse())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get the current value (either the recorded one or the initial value).
    pub fn current_value(&self) -> Option<String> {
        if self.is_recording && !self.keystrokes.is_empty() {
            Some(self.keybinding_string())
        } else {
            self.value.clone()
        }
    }

    /// Check if recording is in progress.
    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    /// Set the value programmatically.
    pub fn set_value(&mut self, value: Option<String>) {
        self.value = value;
    }

    fn start_recording(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.is_recording = true;
        self.keystrokes.clear();
        KeystrokeRecordingState::update(cx, true, Vec::new());
        cx.emit(KeystrokeRecorderEvent::RecordingStarted);
        cx.notify();
    }

    fn cancel_recording(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.is_recording = false;
        self.keystrokes.clear();
        KeystrokeRecordingState::update(cx, false, Vec::new());
        cx.emit(KeystrokeRecorderEvent::RecordingCancelled);
        cx.notify();
    }

    fn confirm_recording(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let keybinding = self.keybinding_string();
        self.is_recording = false;
        self.value = if keybinding.is_empty() {
            None
        } else {
            Some(keybinding.clone())
        };
        self.keystrokes.clear();
        KeystrokeRecordingState::update(cx, false, Vec::new());

        if let Some(callback) = self.on_change.clone() {
            callback(keybinding.clone(), window, cx);
        }

        cx.emit(KeystrokeRecorderEvent::RecordingCompleted(keybinding));
        cx.notify();
    }

    fn handle_key_down(
        &mut self,
        keystroke: &Keystroke,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.is_recording {
            return;
        }

        // Check for cancellation keys: Esc or ctrl-g
        if keystroke.key == "escape" {
            self.cancel_recording(window, cx);
            cx.stop_propagation();
            return;
        }

        if keystroke.key == "g" && keystroke.modifiers.control {
            self.cancel_recording(window, cx);
            cx.stop_propagation();
            return;
        }

        // Check for confirmation key: Enter
        if keystroke.key == "enter" {
            self.confirm_recording(window, cx);
            cx.stop_propagation();
            return;
        }

        // Record the keystroke
        self.keystrokes.push(keystroke.clone());
        KeystrokeRecordingState::update(cx, true, self.keystrokes.clone());
        cx.stop_propagation();
        cx.notify();
    }
}

impl Focusable for KeystrokeRecorder {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for KeystrokeRecorder {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_recording = self.is_recording;
        let display_text = if is_recording {
            if self.keystrokes.is_empty() {
                SharedString::from("")
            } else {
                SharedString::from(self.keybinding_string())
            }
        } else {
            self.value
                .clone()
                .map(SharedString::from)
                .unwrap_or_default()
        };

        let placeholder_visible = display_text.is_empty();
        let placeholder: SharedString = if is_recording {
            t("shortcut.recording_hint")
        } else {
            t("shortcut.click_to_record")
        };

        v_flex()
            .id("keystroke-recorder")
            .track_focus(&self.focus_handle)
            .capture_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                this.handle_key_down(&event.keystroke, window, cx);
            }))
            .on_click(cx.listener(|this, _, window, cx| {
                if !this.is_recording {
                    this.focus_handle.focus(window, cx);
                }
            }))
            .w_full()
            .px_2()
            .py_1()
            .rounded_sm()
            .border_1()
            .border_color(if is_recording {
                cx.theme().colors().border_focused
            } else {
                cx.theme().colors().border
            })
            .bg(cx.theme().colors().editor_background)
            .cursor_pointer()
            .child(
                h_flex()
                    .gap_1()
                    .when(placeholder_visible, |this| {
                        this.child(
                            Label::new(placeholder)
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        )
                    })
                    .when(!placeholder_visible, |this| {
                        this.child(
                            Label::new(display_text)
                                .size(LabelSize::Small)
                                .color(if is_recording {
                                    Color::Accent
                                } else {
                                    Color::Default
                                }),
                        )
                    })
                    .when(is_recording && !self.keystrokes.is_empty(), |this| {
                        this.child(
                            Label::new("...")
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        )
                    }),
            )
    }
}
