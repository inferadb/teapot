//! Comprehensive styling example demonstrating Lip Gloss-style features.
//!
//! Run with: cargo run --example styling

use ferment::style::{
    BorderStyle, Color, Position, Style, join_horizontal, join_horizontal_with, join_vertical,
    join_vertical_with, place, place_horizontal,
};

fn main() {
    println!("\n=== Ferment Styling Demo ===\n");

    // Basic styling
    demo_basic_styling();

    // CSS-like shorthand
    demo_css_shorthand();

    // Borders
    demo_borders();

    // Layout utilities
    demo_layout();

    // Adaptive colors
    demo_adaptive_colors();

    // Style inheritance
    demo_inheritance();

    println!("\n=== End Demo ===\n");
}

fn demo_basic_styling() {
    println!("--- Basic Styling ---\n");

    let bold = Style::new().bold(true).render("Bold text");
    let italic = Style::new().italic(true).render("Italic text");
    let underline = Style::new().underline(true).render("Underlined");
    let colored = Style::new().fg(Color::Cyan).bg(Color::Black).render("Cyan on black");

    println!("{}", bold);
    println!("{}", italic);
    println!("{}", underline);
    println!("{}", colored);
    println!();
}

fn demo_css_shorthand() {
    println!("--- CSS Shorthand ---\n");

    // Padding examples
    let p1 = Style::new().padding(&[1]).bg(Color::Blue).render("Padding: 1 all sides");

    let p2 =
        Style::new().padding(&[0, 2]).bg(Color::Green).render("Padding: 0 vertical, 2 horizontal");

    let p3 = Style::new()
        .padding(&[1, 2, 1, 2])
        .bg(Color::Magenta)
        .render("Padding: 1 top, 2 right, 1 bottom, 2 left");

    println!("{}\n", p1);
    println!("{}\n", p2);
    println!("{}\n", p3);
}

fn demo_borders() {
    println!("--- Border Styles ---\n");

    let rounded =
        Style::new().border(BorderStyle::Rounded).padding(&[0, 1]).render("Rounded border");

    let single = Style::new().border(BorderStyle::Single).padding(&[0, 1]).render("Single border");

    let double = Style::new().border(BorderStyle::Double).padding(&[0, 1]).render("Double border");

    let ascii = Style::new().border(BorderStyle::Ascii).padding(&[0, 1]).render("ASCII border");

    println!("{}\n", rounded);
    println!("{}\n", single);
    println!("{}\n", double);
    println!("{}\n", ascii);
}

fn demo_layout() {
    println!("--- Layout Utilities ---\n");

    // Create some boxes
    let box1 =
        Style::new().border(BorderStyle::Rounded).padding(&[0, 1]).fg(Color::Red).render("Box 1");

    let box2 =
        Style::new().border(BorderStyle::Rounded).padding(&[0, 1]).fg(Color::Green).render("Box 2");

    let box3 =
        Style::new().border(BorderStyle::Rounded).padding(&[0, 1]).fg(Color::Blue).render("Box 3");

    // Horizontal join (default alignment)
    println!("Horizontal join:");
    let horizontal = join_horizontal(&[&box1, &box2, &box3]);
    println!("{}\n", horizontal);

    // Horizontal join with position
    println!("Horizontal join (top-aligned):");
    let horizontal_top = join_horizontal_with(Position::Top, &[&box1, &box2, &box3]);
    println!("{}\n", horizontal_top);

    // Vertical join
    println!("Vertical join:");
    let vertical = join_vertical(&[&box1, &box2, &box3]);
    println!("{}\n", vertical);

    // Vertical join with position
    println!("Vertical join (centered):");
    let vertical_center = join_vertical_with(Position::Center, &[&box1, &box2, &box3]);
    println!("{}\n", vertical_center);

    // Place in a box
    println!("Centered in 40x5 box:");
    let centered = place(40, 5, Position::Center, Position::Center, "Centered!");
    let framed = Style::new().border(BorderStyle::Single).render(&centered);
    println!("{}\n", framed);

    // Horizontal centering
    println!("Horizontally centered in 40 chars:");
    let h_centered = place_horizontal(40, Position::Center, "< centered >");
    println!("|{}|", h_centered);
    println!();
}

fn demo_adaptive_colors() {
    println!("--- Adaptive Colors ---\n");

    // Detect terminal background
    let has_dark = ferment::style::has_dark_background();
    println!("Terminal background: {}", if has_dark { "dark" } else { "light" });

    // Adaptive color (changes based on background)
    let adaptive = Color::Adaptive {
        light: Box::new(Color::Ansi256(236)), // Dark gray for light backgrounds
        dark: Box::new(Color::Ansi256(252)),  // Light gray for dark backgrounds
    };

    let styled = Style::new().fg(adaptive).render("This text adapts to your terminal background");
    println!("{}\n", styled);

    // Complete color (different representations for different terminal types)
    // true_color is a tuple (r, g, b)
    let complete = Color::Complete {
        true_color: (255, 102, 0), // Orange in RGB
        ansi256: 208,
        ansi: 3, // Yellow fallback for 16-color terminals
    };

    let orange = Style::new().fg(complete).bold(true).render("Orange text");
    println!("{}\n", orange);
}

fn demo_inheritance() {
    println!("--- Style Inheritance ---\n");

    // Base style
    let base = Style::new().fg(Color::White).bold(true);

    // Derived styles
    let error = Style::new().inherit(&base).fg(Color::Red);

    let warning = Style::new().inherit(&base).fg(Color::Yellow);

    let success = Style::new().inherit(&base).fg(Color::Green);

    println!("{}", error.render("Error: Something went wrong"));
    println!("{}", warning.render("Warning: Check this out"));
    println!("{}", success.render("Success: All done!"));
    println!();

    // Unsetting properties
    let plain = error.clone().unset_bold().unset_foreground();
    println!("{}", plain.render("Unset bold and color"));
    println!();
}
