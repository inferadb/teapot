# Ferment

A Rust-native terminal UI framework inspired by [Bubble Tea](https://github.com/charmbracelet/bubbletea).

## Overview

`ferment` provides a functional, declarative approach to building terminal user interfaces:

- **Model-Update-View** - Core architecture based on The Elm Architecture
- **Composable Components** - Reusable widgets like spinners, inputs, and selectors
- **Form System** - Declarative form building with validation
- **CI-Friendly** - Automatic non-interactive mode detection

## Quick Start

```rust
use ferment::{Model, Program, Cmd, Event, KeyCode};

struct Counter {
    count: i32,
}

enum Msg {
    Increment,
    Decrement,
    Quit,
}

impl Model for Counter {
    type Message = Msg;

    fn init(&self) -> Option<Cmd<Self::Message>> {
        None
    }

    fn update(&mut self, msg: Self::Message) -> Option<Cmd<Self::Message>> {
        match msg {
            Msg::Increment => self.count += 1,
            Msg::Decrement => self.count -= 1,
            Msg::Quit => return Some(Cmd::quit()),
        }
        None
    }

    fn view(&self) -> String {
        format!("Count: {}\n\nPress +/- to change, q to quit", self.count)
    }

    fn handle_event(&self, event: Event) -> Option<Self::Message> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('+') => Some(Msg::Increment),
                KeyCode::Char('-') => Some(Msg::Decrement),
                KeyCode::Char('q') => Some(Msg::Quit),
                _ => None,
            },
            _ => None,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Program::new(Counter { count: 0 }).run()?;
    Ok(())
}
```

## Components

### Spinner

```rust
use ferment::components::{Spinner, SpinnerStyle};

let spinner = Spinner::new()
    .style(SpinnerStyle::Dots)
    .message("Loading...");
```

### Progress Bar

```rust
use ferment::components::Progress;

let progress = Progress::new()
    .total(100)
    .current(45)
    .message("Downloading...");
```

### Text Input

```rust
use ferment::components::TextInput;

let input = TextInput::new()
    .placeholder("Enter your name...")
    .prompt("> ");
```

### Select

```rust
use ferment::components::Select;

let select = Select::new("Choose a color")
    .options(vec!["Red", "Green", "Blue"]);
```

### Confirm

```rust
use ferment::components::Confirm;

let confirm = Confirm::new("Are you sure?")
    .default(false);
```

## Forms

Build multi-step forms with validation:

```rust
use ferment::forms::{Form, Group, InputField, SelectField, ConfirmField};

let form = Form::new()
    .title("User Registration")
    .group(
        Group::new()
            .title("Personal Info")
            .field(InputField::new("name").title("Your name").required().build())
            .field(InputField::new("email").title("Email").build())
    )
    .group(
        Group::new()
            .title("Preferences")
            .field(SelectField::new("theme").title("Theme")
                .options(["Light", "Dark", "System"]).build())
            .field(ConfirmField::new("newsletter").title("Subscribe?").build())
    );
```

## Styling

```rust
use ferment::style::{Style, Color};

let styled = Style::new()
    .fg(Color::Cyan)
    .bold()
    .render("Hello, World!");
```

## Architecture

The framework follows The Elm Architecture:

1. **Model** - Your application state (any Rust struct)
2. **Message** - Events that trigger state changes
3. **Update** - Pure function that handles messages and updates state
4. **View** - Pure function that renders state as a string
5. **Commands** - Side effects (timers, async operations)

```
┌─────────────────────────────────────────┐
│              Runtime Loop               │
└─────────────────────────────────────────┘
                    │
    ┌───────────────┼───────────────┐
    ▼               ▼               ▼
┌─────────┐   ┌──────────┐   ┌─────────┐
│  Model  │──▶│   View   │   │  Update │
│ (State) │   │ (Render) │   │ (Logic) │
└─────────┘   └──────────┘   └─────────┘
    ▲               │               │
    │        returns String         │
    │               │               │
    └───────────────┴───────────────┘
           New Model + Cmd
```

## CI/Script Compatibility

The framework automatically detects non-interactive environments:

- No animations or spinners
- Clear error messages
- Appropriate exit codes
- Works with piped input/output

## License

Apache-2.0
