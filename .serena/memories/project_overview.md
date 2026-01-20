# Teapot TUI Framework

## Purpose
Teapot (package name: `teapot`) is a Rust-native terminal UI framework following The Elm Architecture, inspired by Go's Bubble Tea library. It provides a functional, declarative approach to building terminal user interfaces.

## Tech Stack
- **Language**: Rust (Edition 2021, MSRV 1.92)
- **Terminal Backend**: crossterm 0.29
- **Unicode Handling**: unicode-width, unicode-segmentation
- **Testing**: tokio (for async tests)

## Core Features
- **Model-Update-View Architecture**: Based on The Elm Architecture
- **Composable Components**: Spinners, text inputs, selectors, tables, viewports, etc.
- **Form System**: Declarative form building with validation
- **CI-Friendly**: Automatic non-interactive mode detection
- **Accessibility**: Screen reader support via ACCESSIBLE=1 environment variable

## Project Structure
```
src/
├── lib.rs              # Main entry point, re-exports
├── output.rs           # Output utilities
├── runtime/            # Core runtime, Program, Commands, Subscriptions
│   ├── mod.rs
│   ├── program.rs      # Main Program runner
│   ├── command.rs      # Command (Cmd) type for side effects
│   ├── message.rs      # Message handling
│   ├── subscription.rs # Subscriptions for events
│   └── accessible.rs   # Accessibility support
├── components/         # UI Components
│   ├── text_input.rs   # Single-line text input
│   ├── text_area.rs    # Multi-line text editor
│   ├── select.rs       # Single-choice selection
│   ├── multi_select.rs # Multiple-choice selection
│   ├── confirm.rs      # Yes/No confirmation
│   ├── list.rs         # Filterable, paginated list
│   ├── spinner.rs      # Loading indicator
│   ├── progress.rs     # Progress bar
│   ├── multi_progress.rs # Parallel progress bars
│   ├── viewport.rs     # Scrollable content container
│   ├── table.rs        # Data table
│   ├── file_picker.rs  # File/directory browser
│   ├── tab_bar.rs      # Tab navigation
│   ├── title_bar.rs    # Title bar component
│   ├── footer_hints.rs # Footer with keybindings
│   ├── status_badge.rs # Status indicator
│   ├── task_list.rs    # Task list display
│   ├── task_progress.rs # Task progress indicator
│   └── modal.rs        # Modal dialog
├── forms/              # Form system
│   ├── form.rs         # Form container
│   ├── group.rs        # Field grouping
│   ├── field.rs        # Field types
│   └── validation.rs   # Validation logic
├── style/              # Styling system (inspired by Lip Gloss)
│   ├── color.rs        # Color definitions
│   ├── border.rs       # Border styles
│   └── text.rs         # Text styling
├── terminal/           # Terminal abstraction
│   ├── backend.rs      # Terminal backend
│   ├── input.rs        # Input handling
│   └── output.rs       # Output handling
└── util/               # Utilities
    ├── keys.rs         # Key handling
    ├── size.rs         # Size utilities
    ├── scroll.rs       # Scroll logic
    └── worker.rs       # Background worker support

examples/               # Example applications
├── form_demo.rs        # Form demonstration
├── layout.rs           # Layout examples
├── styling.rs          # Style examples
└── theming.rs          # Theme examples
```

## Architecture Notes
The framework follows The Elm Architecture:
1. **Model** - Application state (any Rust struct)
2. **Message** - Events that trigger state changes
3. **Update** - Pure function handling messages and updating state
4. **View** - Pure function rendering state as a string
5. **Commands** - Side effects (timers, async operations)

Used by the InferaDB CLI for interactive terminal features.
