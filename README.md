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

Run the following Cargo command in your project directory:

```bash
cargo add teapot
```

## Quick Start

```rust
use teapot::{Model, Program, Cmd, Event, KeyCode};

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

### Text Input

Single-line input with cursor support, placeholders, and password masking.

```rust
use teapot::components::TextInput;

let input = TextInput::new()
    .placeholder("Enter your name...")
    .prompt("> ")
    .build();
```

### Text Area

Multi-line editor with cursor navigation, scrolling, and line editing.

```rust
use teapot::components::TextArea;

let textarea = TextArea::new()
    .placeholder("Enter your message...")
    .height(10)
    .width(60);
```

#### External Editor Support

Open in an external editor with `Ctrl+O`:

```rust
let textarea = TextArea::new()
    .placeholder("Enter code...")
    .editor("code --wait")      // Use VS Code (default: $VISUAL or $EDITOR)
    .editor_extension("rs");    // File extension for syntax highlighting
```

### Selection Components

#### Select

Single-choice selection from a list of options.

```rust
use teapot::components::Select;

let select = Select::new("Choose a color")
    .options(vec!["Red", "Green", "Blue"]);
```

#### MultiSelect

Multiple-choice selection with checkboxes and optional min/max constraints.

```rust
use teapot::components::MultiSelect;

let select = MultiSelect::new("Choose colors")
    .options(vec!["Red", "Green", "Blue"])
    .min(1)
    .max(2);
```

#### Confirm

Yes/No confirmation prompt with customizable default.

```rust
use teapot::components::Confirm;

let confirm = Confirm::new("Are you sure?")
    .default(false);
```

#### List

Filterable, paginated list with keyboard navigation and search.

```rust
use teapot::components::List;

let list = List::new("Select a file")
    .items(vec!["main.rs", "lib.rs", "Cargo.toml"])
    .height(10)
    .filterable(true);
```

### Progress Indicators

#### Spinner

Animated indicator for indeterminate operations.

```rust
use teapot::components::{Spinner, SpinnerStyle};

let spinner = Spinner::new()
    .style(SpinnerStyle::Dots)
    .message("Loading...");
```

#### Progress

Progress bar for determinate operations.

```rust
use teapot::components::Progress;

let progress = Progress::new()
    .total(100)
    .current(45)
    .message("Downloading...");
```

#### MultiProgress

Progress bars for concurrent tasks.

```rust
use teapot::components::MultiProgress;

let mp = MultiProgress::new()
    .add_task("download", "Downloading files...", 100)
    .add_task("compile", "Compiling...", 50)
    .add_task("test", "Running tests...", 200);
```

### Display Components

#### Viewport

Scrollable container with keyboard navigation.

```rust
use teapot::components::Viewport;

let viewport = Viewport::new(80, 20)
    .content("Long scrollable content here...");
```

#### Table

Data table with columns, alignment options, and row selection.

```rust
use teapot::components::{Table, Column};

let table = Table::new()
    .columns(vec![
        Column::new("Name").width(20),
        Column::new("Age").width(5),
        Column::new("City").width(15),
    ])
    .rows(vec![
        vec!["Alice", "30", "New York"],
        vec!["Bob", "25", "Los Angeles"],
    ])
    .height(10);
```

## Forms

Multi-step forms with validation, inspired by [Huh](https://github.com/charmbracelet/huh).

### Basic Form

```rust
use teapot::forms::{Form, Group, Field};

let form = Form::new()
    .title("User Registration")
    .group(
        Group::new()
            .title("Personal Info")
            .field(Field::input().key("name").title("Your name").required(true).build())
            .field(Field::input().key("email").title("Email").build())
    )
    .group(
        Group::new()
            .title("Preferences")
            .field(Field::select()
                .key("theme")
                .title("Theme")
                .options(vec!["Light".into(), "Dark".into(), "System".into()])
                .build())
            .field(Field::confirm().key("newsletter").title("Subscribe?").build())
    );
```

### Form Layouts

Form group display options:

```rust
use teapot::forms::{Form, FormLayout};

// One group at a time (wizard-style)
let wizard = Form::new().layout(FormLayout::Default);

// Stack: all groups visible at once
let stacked = Form::new().layout(FormLayout::Stack);

// Columns: side-by-side layout
let columns = Form::new().layout(FormLayout::Columns(2));
```

### All Field Types

```rust
use teapot::forms::Field;

// Text input with validation
Field::input()
    .key("email")
    .title("Email Address")
    .placeholder("user@example.com")
    .required(true)
    .build();

// Single selection
Field::select()
    .key("country")
    .title("Country")
    .options(vec!["USA".into(), "Canada".into(), "UK".into(), "Germany".into()])
    .build();

// Multiple selection with constraints
Field::multi_select()
    .key("languages")
    .title("Languages")
    .options(vec!["Rust".into(), "Go".into(), "Python".into(), "TypeScript".into()])
    .min(1)
    .max(3)
    .build();

// Yes/No confirmation
Field::confirm()
    .key("agree")
    .title("Accept terms?")
    .default(false)
    .build();

// Display-only note
Field::note()
    .content("Please review carefully before proceeding.")
    .title("Important")
    .build();

// File/directory picker
Field::file_picker()
    .key("config_file")
    .title("Select config file")
    .directory(std::path::PathBuf::from("/etc"))
    .extensions(vec!["toml".into(), "yaml".into(), "json".into()])
    .build();
```

### Dynamic Content

Field titles and descriptions can update dynamically:

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use teapot::forms::Field;

let attempt = Arc::new(AtomicUsize::new(1));
let attempt_clone = attempt.clone();

Field::input()
    .key("password")
    .title_fn(Arc::new(move || format!("Password (attempt {})", attempt_clone.load(Ordering::SeqCst))))
    .description_fn(Arc::new(|| "Must be at least 8 characters".to_string()))
    .build();
```

### FilePicker Component

Browse and select files/directories:

```rust
use teapot::components::FilePicker;

let picker = FilePicker::new()
    .title("Select a file")
    .directory("/home/user/projects")
    .extensions(["rs", "toml"])  // Filter by extension
    .show_hidden(false)          // Hide dotfiles
    .height(15);                 // Visible rows

// Or for directory selection only
let dir_picker = FilePicker::new()
    .title("Select output directory")
    .dirs_only();
```

## Styling

Styling system inspired by [Lip Gloss](https://github.com/charmbracelet/lipgloss).

### Basic Styling

```rust
use teapot::style::{Style, Color, BorderStyle};

let styled = Style::new()
    .foreground(Color::Cyan)
    .background(Color::Black)
    .bold(true)
    .italic(true)
    .border(BorderStyle::Rounded)
    .render("Hello, World!");
```

### CSS-like Shorthand

Padding and margin use CSS shorthand (1-4 values):

```rust
use teapot::style::Style;

// All sides: 2
Style::new().padding(&[2]);

// Vertical: 1, Horizontal: 2
Style::new().padding(&[1, 2]);

// Top: 1, Horizontal: 2, Bottom: 3
Style::new().margin(&[1, 2, 3]);

// Top: 1, Right: 2, Bottom: 3, Left: 4 (clockwise)
Style::new().margin(&[1, 2, 3, 4]);
```

### Block Dimensions

Control width, height, and alignment:

```rust
use teapot::style::{Style, BorderStyle, Position};

let box_style = Style::new()
    .width(40)
    .height(10)
    .max_width(80)
    .align(Position::Center, Position::Center)
    .border(BorderStyle::Rounded);
```

### Layout Utilities

Compose blocks horizontally or vertically:

```rust
use teapot::style::{join_horizontal_with, join_vertical_with, place, Position};

// Side-by-side blocks (aligned at top)
let combined = join_horizontal_with(Position::Top, &[left_block, right_block]);

// Stacked blocks (centered horizontally)
let stacked = join_vertical_with(Position::Center, &[header, content, footer]);

// Position content in a box
let centered = place(80, 24, Position::Center, Position::Center, "Centered!");
```

### Adaptive Colors

Colors that adapt to light/dark terminal backgrounds:

```rust
use teapot::style::Color;

// Different colors for light vs dark backgrounds
let adaptive = Color::Adaptive {
    light: Box::new(Color::Ansi256(236)),  // Dark gray for light bg
    dark: Box::new(Color::Ansi256(252)),   // Light gray for dark bg
};

// Full color specification for all terminal types
let complete = Color::Complete {
    true_color: (255, 102, 0),  // RGB tuple
    ansi256: 208,
    ansi: 3,  // Yellow fallback
};
```

### Style Inheritance

Build styles incrementally:

```rust
use teapot::style::{Style, Color};

let base = Style::new()
    .foreground(Color::White)
    .bold(true);

let highlight = Style::new()
    .background(Color::Blue)
    .inherit(&base);       // Copy unset properties from base

// Unset specific properties
let plain = highlight.unset_bold().unset_background();
```

## Program Configuration

### Builder Pattern

Builder API configuration:

```rust
use teapot::{Program, Model};
use std::time::Duration;

Program::new(my_model)
    .with_alt_screen()           // Use alternate screen buffer
    .with_mouse()                // Enable mouse events
    .with_bracketed_paste()      // Enable paste detection
    .with_focus_change()         // Enable focus/blur events
    .with_tick_rate(Duration::from_millis(16))  // ~60 FPS
    .with_accessible()           // Force accessible mode
    .run()?;
```

### Message Filtering

Pre-process or block messages before update:

```rust
Program::new(my_model)
    .with_filter(|model, msg| {
        // Block all messages while loading
        if model.is_loading {
            return None;
        }
        // Transform or pass through
        Some(msg)
    })
    .run()?;
```

### Commands

Bubble Tea-style commands via the `cmd` module:

```rust
use teapot::cmd;
use std::time::Duration;

// Quit the program
cmd::quit()

// Batch multiple commands
cmd::batch(vec![cmd1, cmd2, cmd3])

// Sequential execution
cmd::sequence(vec![cmd1, cmd2, cmd3])

// Periodic tick
cmd::tick(Duration::from_secs(1), |_| Msg::Tick)

// No-op command
cmd::none()
```

### External Process Execution

Spawn external processes with terminal teardown/restore:

```rust
use teapot::Cmd;
use std::process::Command;

enum Msg {
    EditorClosed(bool),
    EditorFailed,
}

let mut command = Command::new("vim");
command.arg("file.txt");

let cmd: Cmd<Msg> = Cmd::run_process(command, |result| {
    match result {
        Ok(status) => Msg::EditorClosed(status.success()),
        Err(_) => Msg::EditorFailed,
    }
});
```

## Architecture

Follows The Elm Architecture:

1. **Model** - Application state
2. **Message** - Events triggering state changes
3. **Update** - Handles messages, updates state
4. **View** - Renders state to string
5. **Commands** - Side effects (timers, async I/O)

```mermaid
flowchart TD
    subgraph Runtime["Runtime Loop"]
        direction LR
        Model["Model<br/>(State)"]
        View["View<br/>(Render)"]
        Update["Update<br/>(Logic)"]
    end

    Model --> View
    View -->|"returns String"| Terminal["Terminal Output"]
    Events["User Events"] --> Update
    Update -->|"New Model + Cmd"| Model

    subgraph Commands["Commands (Effects)"]
        Tick["Tick timers"]
        Async["Async I/O"]
        Quit["Quit signal"]
    end

    Update --> Commands
    Commands --> Update
```

## CI/Script Compatibility

Automatically detects non-interactive environments:

- No animations
- Clear error messages
- Proper exit codes
- Piped I/O support

## Accessibility

Accessible mode supports screen readers and assistive technologies.

### Enabling Accessible Mode

Set the `ACCESSIBLE` environment variable:

```bash
ACCESSIBLE=1 ./my-app
```

### What Changes in Accessible Mode

- **Plain text** - No ANSI codes or formatting
- **Numbered options** - Numbers replace arrow navigation
- **Line-based input** - Standard stdin instead of raw mode
- **Clear prompts** - Screen reader-friendly descriptions

### Accessible Forms

Use `Form::run_accessible()` for a fully accessible form experience:

```rust
use teapot::forms::{Form, Group, Field};

let mut form = Form::new()
    .title("User Survey")
    .group(
        Group::new()
            .field(Field::input().key("name").title("Your name").build())
            .field(Field::select()
                .key("color")
                .title("Favorite color")
                .options(vec!["Red".into(), "Green".into(), "Blue".into()])
                .build())
            .field(Field::confirm().key("subscribe").title("Subscribe to newsletter?").build())
    );

// Run in accessible mode (line-based prompts)
match form.run_accessible() {
    Ok(Some(results)) => {
        println!("Name: {}", results.get_string("name").unwrap_or(""));
        println!("Color: {}", results.get_string("color").unwrap_or(""));
        println!("Subscribe: {}", results.get_bool("subscribe").unwrap_or(false));
    }
    Ok(None) => println!("Form cancelled"),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Example Accessible Session

```
=== User Survey ===

Your name
?
> Alice

Favorite color
? Favorite color
  1) Red
  2) Green
* 3) Blue
Enter number (or q to cancel): 3

Subscribe to newsletter?
? Subscribe to newsletter? (y/N) y

Form completed!
```

### Custom Accessible Components

Implement the `Accessible` trait for custom rendering:

```rust
use teapot::{Accessible, Model};

impl Accessible for MyComponent {
    type Message = MyMsg;

    fn accessible_prompt(&self) -> String {
        // Return plain text prompt
        format!("? {}\n> ", self.title)
    }

    fn parse_accessible_input(&self, input: &str) -> Option<Self::Message> {
        // Parse line input and return message
        Some(MyMsg::SetValue(input.trim().to_string()))
    }

    fn is_accessible_complete(&self) -> bool {
        self.submitted
    }
}
```

### Environment Variables

| Variable          | Description                              |
| ----------------- | ---------------------------------------- |
| `ACCESSIBLE=1`    | Enable accessible mode                   |
| `NO_COLOR=1`      | Disable colors (respected automatically) |
| `REDUCE_MOTION=1` | Disable animations                       |

## Community

Join us on [Discord](https://discord.gg/inferadb) for questions, discussions, and contributions.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE).
