use std::collections::VecDeque;

use getset::Getters;
use nanoid::nanoid;
use serde::Deserialize;
use serde::Serialize;

use crate::ring::Ring;
use crate::window::Window;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Getters)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Container {
    #[getset(get = "pub")]
    id: String,
    windows: Ring<Window>,
}

impl_ring_elements!(Container, Window);

impl Default for Container {
    fn default() -> Self {
        Self {
            id: nanoid!(),
            windows: Ring::default(),
        }
    }
}

impl Container {
    pub fn hide(&self, omit: Option<isize>) {
        for window in self.windows().iter().rev() {
            let mut should_hide = omit.is_none();

            if !should_hide {
                if let Some(omit) = omit {
                    if omit != window.hwnd {
                        should_hide = true
                    }
                }
            }

            if should_hide {
                window.hide();
            }
        }
    }

    pub fn restore(&self) {
        if let Some(window) = self.focused_window() {
            window.restore();
        }
    }

    /// Hides the unfocused windows of the container and restores the focused one. This function
    /// is used to make sure we update the window that should be shown on a stack. If the container
    /// isn't a stack this function won't change anything.
    pub fn load_focused_window(&mut self) {
        let focused_idx = self.focused_window_idx();

        for (i, window) in self.windows_mut().iter_mut().enumerate() {
            if i == focused_idx {
                window.restore_with_border(false);
            } else {
                window.hide_with_border(false);
            }
        }
    }

    pub fn hwnd_from_exe(&self, exe: &str) -> Option<isize> {
        for window in self.windows() {
            if let Ok(window_exe) = window.exe() {
                if exe == window_exe {
                    return Option::from(window.hwnd);
                }
            }
        }

        None
    }

    pub fn idx_from_exe(&self, exe: &str) -> Option<usize> {
        for (idx, window) in self.windows().iter().enumerate() {
            if let Ok(window_exe) = window.exe() {
                if exe == window_exe {
                    return Option::from(idx);
                }
            }
        }

        None
    }

    pub fn contains_window(&self, hwnd: isize) -> bool {
        for window in self.windows() {
            if window.hwnd == hwnd {
                return true;
            }
        }

        false
    }

    pub fn idx_for_window(&self, hwnd: isize) -> Option<usize> {
        for (i, window) in self.windows().iter().enumerate() {
            if window.hwnd == hwnd {
                return Option::from(i);
            }
        }

        None
    }

    pub fn remove_window_by_idx(&mut self, idx: usize) -> Option<Window> {
        let window = self.windows_mut().remove(idx);
        self.focus_window(idx.saturating_sub(1));
        window
    }

    pub fn remove_focused_window(&mut self) -> Option<Window> {
        let focused_idx = self.focused_window_idx();
        self.remove_window_by_idx(focused_idx)
    }

    pub fn add_window(&mut self, window: Window) {
        self.windows_mut().push_back(window);
        self.focus_window(self.windows().len().saturating_sub(1));
        let focused_window_idx = self.focused_window_idx();

        for (i, window) in self.windows().iter().enumerate() {
            if i != focused_window_idx {
                window.hide();
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn focus_window(&mut self, idx: usize) {
        tracing::info!("focusing window");
        self.windows.focus(idx);
    }
}
