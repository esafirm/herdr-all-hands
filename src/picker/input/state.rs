use super::PickerInputEvent;

/// Result of feeding one key into the fixed-width hint state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InputDecision {
    Continue,
    Cancel,
    CopyHint(String),
    InvalidHint,
}

/// Pure fixed-width hint buffer for picker input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InputState {
    width: usize,
    buffer: String,
}

impl InputState {
    pub(crate) fn new(width: usize) -> Self {
        Self {
            width,
            buffer: String::new(),
        }
    }

    /// Hint characters typed so far toward the current selection.
    pub(crate) fn typed(&self) -> &str {
        &self.buffer
    }

    /// Feeds one key. A partial prefix that no valid hint starts with is
    /// rejected immediately rather than after the full hint width.
    pub(crate) fn push(&mut self, event: PickerInputEvent, valid_hints: &[&str]) -> InputDecision {
        match event {
            PickerInputEvent::Escape | PickerInputEvent::CtrlC => InputDecision::Cancel,
            PickerInputEvent::Enter | PickerInputEvent::Other => InputDecision::Continue,
            PickerInputEvent::Char(ch) => {
                if self.width == 0 {
                    return InputDecision::Continue;
                }

                self.buffer.push(ch);
                if self.buffer.chars().count() < self.width {
                    if valid_hints
                        .iter()
                        .any(|hint| hint.starts_with(&self.buffer))
                    {
                        return InputDecision::Continue;
                    }
                    self.buffer.clear();
                    return InputDecision::InvalidHint;
                }

                let entered = std::mem::take(&mut self.buffer);
                if valid_hints.iter().any(|hint| *hint == entered) {
                    InputDecision::CopyHint(entered)
                } else {
                    InputDecision::InvalidHint
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_one_character_hint_requests_copy() {
        let mut state = InputState::new(1);

        assert_eq!(
            state.push(PickerInputEvent::Char('a'), &["a"]),
            InputDecision::CopyHint("a".to_string())
        );
    }

    #[test]
    fn exact_two_character_hint_waits_for_full_width() {
        let mut state = InputState::new(2);

        assert_eq!(
            state.push(PickerInputEvent::Char('a'), &["as"]),
            InputDecision::Continue
        );
        assert_eq!(
            state.push(PickerInputEvent::Char('s'), &["as"]),
            InputDecision::CopyHint("as".to_string())
        );
    }

    #[test]
    fn invalid_full_width_hint_clears_buffer() {
        let mut state = InputState::new(2);

        assert_eq!(
            state.push(PickerInputEvent::Char('s'), &["sd"]),
            InputDecision::Continue
        );
        assert_eq!(
            state.push(PickerInputEvent::Char('x'), &["sd"]),
            InputDecision::InvalidHint
        );
        assert_eq!(state.typed(), "");
        assert_eq!(
            state.push(PickerInputEvent::Char('s'), &["sd"]),
            InputDecision::Continue
        );
        assert_eq!(state.typed(), "s");
        assert_eq!(
            state.push(PickerInputEvent::Char('d'), &["sd"]),
            InputDecision::CopyHint("sd".to_string())
        );
    }

    #[test]
    fn unmatched_first_key_resets_immediately() {
        let mut state = InputState::new(2);

        assert_eq!(
            state.push(PickerInputEvent::Char('a'), &["sd"]),
            InputDecision::InvalidHint
        );
        assert_eq!(state.typed(), "");
    }

    #[test]
    fn escape_and_ctrl_c_cancel() {
        let mut state = InputState::new(1);
        assert_eq!(
            state.push(PickerInputEvent::Escape, &[]),
            InputDecision::Cancel
        );
        assert_eq!(
            state.push(PickerInputEvent::CtrlC, &[]),
            InputDecision::Cancel
        );
    }

    #[test]
    fn enter_is_ignored() {
        let mut state = InputState::new(1);
        assert_eq!(
            state.push(PickerInputEvent::Enter, &["a"]),
            InputDecision::Continue
        );
    }
}
