<div align="center">
    <p><a href="https://inferadb.com"><img src=".github/inferadb.png" width="100" /></a></p>
    <h1>Teapot</h1>
    <p>
        <a href="https://discord.gg/inferadb"><img src="https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white" alt="Discord" /></a>
        <a href="#license"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg" alt="License" /></a>
    </p>
    <p>A Rust Terminal UI framework inspired by Bubble Tea</p>
</div>

> [!IMPORTANT]
> Under active development. Not production-ready.

`teapot` is a functional, declarative TUI framework:

- **Model-Update-View** - Core architecture based on The Elm Architecture
- **Composable Components** - Reusable widgets like spinners, inputs, and selectors
- **Form System** - Declarative form building with validation
- **CI-Friendly** - Automatic non-interactive mode detection

## Installation

```bash
cargo add teapot
```

## Quick Start

```rust
use teapot::{Model, Program, Cmd, Event, KeyCode};

struct Counter { count: i32 }

enum Msg { Increment, Decrement, Quit }

impl Model for Counter {
    type Message = Msg;

    fn init(&self) -> Option<Cmd<Self::Message>> { None }

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
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('+') => return Some(Msg::Increment),
                KeyCode::Char('-') => return Some(Msg::Decrement),
                KeyCode::Char('q') => return Some(Msg::Quit),
                _ => {}
            }
        }
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Program::new(Counter { count: 0 }).run()
}
```

## Components

| Component | Description |
|-----------|-------------|
| `TextInput` | Single-line input with cursor, placeholder, password masking |
| `TextArea` | Multi-line editor with external editor support (`Ctrl+O`) |
| `Select` | Single-choice selection |
| `MultiSelect` | Multiple-choice with min/max constraints |
| `Confirm` | Yes/No prompt |
| `List` | Filterable, paginated list |
| `Spinner` | Animated loading indicator |
| `Progress` | Progress bar |
| `MultiProgress` | Concurrent task progress |
| `Viewport` | Scrollable container |
| `Table` | Data table with columns and selection |
| `FilePicker` | File/directory browser |

See `examples/` for usage patterns.

## Forms

Multi-step forms with validation, inspired by [Huh](https://github.com/charmbracelet/huh).

```rust
use teapot::forms::{Form, Group, Field};

let mut form = Form::new()
    .title("User Registration")
    .group(
        Group::new()
            .title("Personal Info")
            .field(Field::input().key("name").title("Name").required(true).build())
            .field(Field::select()
                .key("theme")
                .title("Theme")
                .options(vec!["Light".into(), "Dark".into()])
                .build())
            .field(Field::confirm().key("subscribe").title("Subscribe?").build())
    );

let results = form.run_accessible()?;
```

**Field types:** `input`, `select`, `multi_select`, `confirm`, `note`, `file_picker`

**Layouts:** `FormLayout::Default` (wizard), `FormLayout::Stack`, `FormLayout::Columns(n)`

## Styling

Styling system inspired by [Lip Gloss](https://github.com/charmbracelet/lipgloss).

```rust
use teapot::style::{Style, Color, BorderStyle};

let styled = Style::new()
    .foreground(Color::Cyan)
    .bold(true)
    .border(BorderStyle::Rounded)
    .padding(&[1, 2])  // CSS shorthand: vertical, horizontal
    .render("Hello!");
```

**Layout utilities:** `join_horizontal_with`, `join_vertical_with`, `place`

**Adaptive colors:** `Color::Adaptive { light, dark }` for terminal background detection

## Architecture

Follows The Elm Architecture:

```
┌─────────────────────────────────────────────────┐
│                    Runtime                       │
│  Model ──► View ──► Terminal                    │
│    ▲                                            │
│    └─── Update ◄─── Events                      │
│              │                                  │
│              └─── Commands (effects)            │
└─────────────────────────────────────────────────┘
```

## Program Configuration

```rust
Program::new(model)
    .with_alt_screen()      // Alternate screen buffer
    .with_mouse()           // Mouse events
    .with_tick_rate(Duration::from_millis(16))  // ~60 FPS
    .run()?;
```

## Accessibility

Set `ACCESSIBLE=1` for screen reader support:

- Plain text output (no ANSI codes)
- Numbered options instead of arrow navigation
- Line-based input

| Variable | Description |
|----------|-------------|
| `ACCESSIBLE=1` | Enable accessible mode |
| `NO_COLOR=1` | Disable colors |
| `REDUCE_MOTION=1` | Disable animations |

## Development

```bash
just          # Run all checks
just test     # Tests
just lint     # Clippy
just fmt      # Format
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Community

Join us on [Discord](https://discord.gg/inferadb).

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE).
