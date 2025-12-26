//! Input event types.
//!
//! This module provides unified event types that wrap crossterm events.

use crossterm::event::{
    Event as CrosstermEvent, KeyCode as CrosstermKeyCode, KeyEvent as CrosstermKeyEvent,
    KeyModifiers as CrosstermKeyModifiers, MouseButton as CrosstermMouseButton,
    MouseEvent as CrosstermMouseEvent, MouseEventKind as CrosstermMouseEventKind,
};

/// A terminal event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A keyboard event.
    Key(KeyEvent),
    /// A mouse event.
    Mouse(MouseEvent),
    /// The terminal was resized.
    Resize { width: u16, height: u16 },
    /// Focus was gained.
    FocusGained,
    /// Focus was lost.
    FocusLost,
    /// Pasted text.
    Paste(String),
}

impl From<CrosstermEvent> for Event {
    fn from(event: CrosstermEvent) -> Self {
        match event {
            CrosstermEvent::Key(key) => Event::Key(KeyEvent::from(key)),
            CrosstermEvent::Mouse(mouse) => Event::Mouse(MouseEvent::from(mouse)),
            CrosstermEvent::Resize(width, height) => Event::Resize { width, height },
            CrosstermEvent::FocusGained => Event::FocusGained,
            CrosstermEvent::FocusLost => Event::FocusLost,
            CrosstermEvent::Paste(text) => Event::Paste(text),
        }
    }
}

/// A keyboard event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    /// The key code.
    pub code: KeyCode,
    /// Modifier keys held.
    pub modifiers: KeyModifiers,
}

impl From<CrosstermKeyEvent> for KeyEvent {
    fn from(event: CrosstermKeyEvent) -> Self {
        Self {
            code: KeyCode::from(event.code),
            modifiers: KeyModifiers::from(event.modifiers),
        }
    }
}

/// Key codes for keyboard events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// A character key.
    Char(char),
    /// Function keys F1-F12.
    F(u8),
    /// Backspace key.
    Backspace,
    /// Enter/Return key.
    Enter,
    /// Left arrow key.
    Left,
    /// Right arrow key.
    Right,
    /// Up arrow key.
    Up,
    /// Down arrow key.
    Down,
    /// Home key.
    Home,
    /// End key.
    End,
    /// Page up key.
    PageUp,
    /// Page down key.
    PageDown,
    /// Tab key.
    Tab,
    /// Backtab (Shift+Tab).
    BackTab,
    /// Delete key.
    Delete,
    /// Insert key.
    Insert,
    /// Escape key.
    Esc,
    /// Caps lock key.
    CapsLock,
    /// Scroll lock key.
    ScrollLock,
    /// Num lock key.
    NumLock,
    /// Print screen key.
    PrintScreen,
    /// Pause key.
    Pause,
    /// Menu key.
    Menu,
    /// Null (no key).
    Null,
}

impl From<CrosstermKeyCode> for KeyCode {
    fn from(code: CrosstermKeyCode) -> Self {
        match code {
            CrosstermKeyCode::Char(c) => KeyCode::Char(c),
            CrosstermKeyCode::F(n) => KeyCode::F(n),
            CrosstermKeyCode::Backspace => KeyCode::Backspace,
            CrosstermKeyCode::Enter => KeyCode::Enter,
            CrosstermKeyCode::Left => KeyCode::Left,
            CrosstermKeyCode::Right => KeyCode::Right,
            CrosstermKeyCode::Up => KeyCode::Up,
            CrosstermKeyCode::Down => KeyCode::Down,
            CrosstermKeyCode::Home => KeyCode::Home,
            CrosstermKeyCode::End => KeyCode::End,
            CrosstermKeyCode::PageUp => KeyCode::PageUp,
            CrosstermKeyCode::PageDown => KeyCode::PageDown,
            CrosstermKeyCode::Tab => KeyCode::Tab,
            CrosstermKeyCode::BackTab => KeyCode::BackTab,
            CrosstermKeyCode::Delete => KeyCode::Delete,
            CrosstermKeyCode::Insert => KeyCode::Insert,
            CrosstermKeyCode::Esc => KeyCode::Esc,
            CrosstermKeyCode::CapsLock => KeyCode::CapsLock,
            CrosstermKeyCode::ScrollLock => KeyCode::ScrollLock,
            CrosstermKeyCode::NumLock => KeyCode::NumLock,
            CrosstermKeyCode::PrintScreen => KeyCode::PrintScreen,
            CrosstermKeyCode::Pause => KeyCode::Pause,
            CrosstermKeyCode::Menu => KeyCode::Menu,
            CrosstermKeyCode::Null => KeyCode::Null,
            _ => KeyCode::Null,
        }
    }
}

/// Modifier keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyModifiers {
    bits: u8,
}

impl KeyModifiers {
    /// Shift key.
    pub const SHIFT: Self = Self { bits: 0b0000_0001 };
    /// Control key.
    pub const CONTROL: Self = Self { bits: 0b0000_0010 };
    /// Alt key.
    pub const ALT: Self = Self { bits: 0b0000_0100 };
    /// Super/Meta/Windows key.
    pub const SUPER: Self = Self { bits: 0b0000_1000 };
    /// Hyper key.
    pub const HYPER: Self = Self { bits: 0b0001_0000 };
    /// Meta key.
    pub const META: Self = Self { bits: 0b0010_0000 };
    /// No modifiers.
    pub const NONE: Self = Self { bits: 0 };

    /// Check if this modifier set contains the given modifier.
    pub fn contains(&self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    /// Check if shift is held.
    pub fn shift(&self) -> bool {
        self.contains(Self::SHIFT)
    }

    /// Check if control is held.
    pub fn ctrl(&self) -> bool {
        self.contains(Self::CONTROL)
    }

    /// Check if alt is held.
    pub fn alt(&self) -> bool {
        self.contains(Self::ALT)
    }

    /// Check if no modifiers are held.
    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }
}

impl From<CrosstermKeyModifiers> for KeyModifiers {
    fn from(mods: CrosstermKeyModifiers) -> Self {
        let mut bits = 0u8;
        if mods.contains(CrosstermKeyModifiers::SHIFT) {
            bits |= Self::SHIFT.bits;
        }
        if mods.contains(CrosstermKeyModifiers::CONTROL) {
            bits |= Self::CONTROL.bits;
        }
        if mods.contains(CrosstermKeyModifiers::ALT) {
            bits |= Self::ALT.bits;
        }
        if mods.contains(CrosstermKeyModifiers::SUPER) {
            bits |= Self::SUPER.bits;
        }
        if mods.contains(CrosstermKeyModifiers::HYPER) {
            bits |= Self::HYPER.bits;
        }
        if mods.contains(CrosstermKeyModifiers::META) {
            bits |= Self::META.bits;
        }
        Self { bits }
    }
}

impl std::ops::BitOr for KeyModifiers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits | rhs.bits,
        }
    }
}

/// A mouse event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseEvent {
    /// The kind of mouse event.
    pub kind: MouseEventKind,
    /// The column position.
    pub column: u16,
    /// The row position.
    pub row: u16,
    /// Modifier keys held.
    pub modifiers: KeyModifiers,
}

impl From<CrosstermMouseEvent> for MouseEvent {
    fn from(event: CrosstermMouseEvent) -> Self {
        Self {
            kind: MouseEventKind::from(event.kind),
            column: event.column,
            row: event.row,
            modifiers: KeyModifiers::from(event.modifiers),
        }
    }
}

/// The kind of mouse event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventKind {
    /// A button was pressed.
    Down(MouseButton),
    /// A button was released.
    Up(MouseButton),
    /// The mouse was dragged with a button held.
    Drag(MouseButton),
    /// The mouse was moved.
    Moved,
    /// The scroll wheel was scrolled down.
    ScrollDown,
    /// The scroll wheel was scrolled up.
    ScrollUp,
    /// The scroll wheel was scrolled left.
    ScrollLeft,
    /// The scroll wheel was scrolled right.
    ScrollRight,
}

impl From<CrosstermMouseEventKind> for MouseEventKind {
    fn from(kind: CrosstermMouseEventKind) -> Self {
        match kind {
            CrosstermMouseEventKind::Down(btn) => MouseEventKind::Down(MouseButton::from(btn)),
            CrosstermMouseEventKind::Up(btn) => MouseEventKind::Up(MouseButton::from(btn)),
            CrosstermMouseEventKind::Drag(btn) => MouseEventKind::Drag(MouseButton::from(btn)),
            CrosstermMouseEventKind::Moved => MouseEventKind::Moved,
            CrosstermMouseEventKind::ScrollDown => MouseEventKind::ScrollDown,
            CrosstermMouseEventKind::ScrollUp => MouseEventKind::ScrollUp,
            CrosstermMouseEventKind::ScrollLeft => MouseEventKind::ScrollLeft,
            CrosstermMouseEventKind::ScrollRight => MouseEventKind::ScrollRight,
        }
    }
}

/// Mouse buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Left mouse button.
    Left,
    /// Right mouse button.
    Right,
    /// Middle mouse button.
    Middle,
}

impl From<CrosstermMouseButton> for MouseButton {
    fn from(btn: CrosstermMouseButton) -> Self {
        match btn {
            CrosstermMouseButton::Left => MouseButton::Left,
            CrosstermMouseButton::Right => MouseButton::Right,
            CrosstermMouseButton::Middle => MouseButton::Middle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_modifiers() {
        let mods = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
        assert!(mods.contains(KeyModifiers::CONTROL));
        assert!(mods.contains(KeyModifiers::SHIFT));
        assert!(!mods.contains(KeyModifiers::ALT));
        assert!(mods.ctrl());
        assert!(mods.shift());
        assert!(!mods.alt());
    }

    #[test]
    fn test_key_code() {
        let code = KeyCode::Char('a');
        assert_eq!(code, KeyCode::Char('a'));

        let code = KeyCode::Enter;
        assert_eq!(code, KeyCode::Enter);
    }
}
