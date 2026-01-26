//! Layout composition example demonstrating Lip Gloss-style layout utilities.
//!
//! Run with: cargo run --example layout

use teapot::style::{
    BorderStyle, Color, Position, Style, join_horizontal, join_horizontal_with, join_vertical_with,
    place, place_horizontal, place_vertical,
};

fn main() {
    println!("\n=== Teapot Layout Composition Demo ===\n");

    // Dashboard-style layout
    demo_dashboard();

    // Card grid
    demo_card_grid();

    // Status bar
    demo_status_bar();

    // Dialog box
    demo_dialog();

    println!("=== End Demo ===\n");
}

fn demo_dashboard() {
    println!("--- Dashboard Layout ---\n");

    // Create sidebar
    let sidebar_style =
        Style::builder().foreground(Color::Cyan).width(20).build().border(BorderStyle::Rounded);

    let sidebar_content = "Navigation\n\n• Dashboard\n• Settings\n• Profile\n• Help\n• Logout";
    let sidebar = sidebar_style.render(sidebar_content);

    // Create main content area
    let main_style = Style::builder().foreground(Color::White).build().border(BorderStyle::Rounded);

    let main_content = "Welcome to Teapot!\n\n\
        This is the main content area.\n\
        It can contain any text or\n\
        rendered components.\n\n\
        Use layout utilities to\n\
        compose complex UIs.";
    let main_area = main_style.render(main_content);

    // Create stats panel
    let stats_style =
        Style::builder().foreground(Color::Green).width(20).build().border(BorderStyle::Single);

    let stats_content = "Statistics\n\n↑ 42% Traffic\n↓ 12% Errors\n→ 1.2k Users";
    let stats = stats_style.render(stats_content);

    // Compose: sidebar | main | stats
    let dashboard = join_horizontal_with(Position::Top, &[&sidebar, &main_area, &stats]);
    println!("{}\n", dashboard);
}

fn demo_card_grid() {
    println!("--- Card Grid ---\n");

    // Create cards
    let card_style =
        Style::builder().width(18).build().border(BorderStyle::Rounded).padding(&[0, 1]);

    let card1 = card_style.clone().fg(Color::Red).render("Error\n━━━━━━━━\n23 issues");

    let card2 = card_style.clone().fg(Color::Yellow).render("Warning\n━━━━━━━━\n12 alerts");

    let card3 = card_style.clone().fg(Color::Green).render("Success\n━━━━━━━━\n156 passed");

    let card4 = card_style.clone().fg(Color::Blue).render("Info\n━━━━━━━━\n8 notes");

    // Arrange in 2x2 grid
    let row1 = join_horizontal_with(Position::Top, &[&card1, &card2]);
    let row2 = join_horizontal_with(Position::Top, &[&card3, &card4]);
    let grid = join_vertical_with(Position::LEFT, &[&row1, &row2]);

    println!("{}\n", grid);
}

fn demo_status_bar() {
    println!("--- Status Bar ---\n");

    let width = 60;

    // Left section: mode indicator
    let mode = Style::builder()
        .background(Color::Blue)
        .foreground(Color::White)
        .bold(true)
        .build()
        .padding(&[0, 1])
        .render(" NORMAL ");

    // Center section: filename
    let filename = "src/main.rs [+]";

    // Right section: position
    let position = Style::builder()
        .background(Color::BrightBlack)
        .foreground(Color::White)
        .build()
        .padding(&[0, 1])
        .render(" Ln 42, Col 15 ");

    // Calculate center padding
    let mode_width = 10; // approximate
    let pos_width = 15; // approximate
    let center_width = width - mode_width - pos_width;
    let centered_filename = place_horizontal(center_width, Position::Center, filename);

    // Compose status bar
    let status_bar = format!("{}{}{}", mode, centered_filename, position);

    // Add a border around it
    let bar_style = Style::builder().width(width).build().border(BorderStyle::Single);
    println!("{}\n", bar_style.render(&status_bar));
}

fn demo_dialog() {
    println!("--- Dialog Box ---\n");

    // Dialog content
    let title = Style::builder().bold(true).build().fg(Color::Cyan).render("Confirm Action");

    let message = "Are you sure you want to delete\nthis file? This cannot be undone.";

    let buttons = {
        let cancel = Style::new().border(BorderStyle::Rounded).padding(&[0, 2]).render("Cancel");

        let confirm = Style::builder()
            .background(Color::Red)
            .foreground(Color::White)
            .bold(true)
            .build()
            .border(BorderStyle::Rounded)
            .padding(&[0, 2])
            .render("Delete");

        join_horizontal_with(Position::Center, &[&cancel, &confirm])
    };

    // Stack content vertically with spacing
    let spacer = "";
    let content =
        join_vertical_with(Position::Center, &[&title, spacer, message, spacer, &buttons]);

    // Center in a box
    let centered = place(50, 10, Position::Center, Position::Center, &content);

    // Add dialog border
    let dialog = Style::new().border(BorderStyle::Double).fg(Color::White).render(&centered);

    println!("{}\n", dialog);
}

#[allow(dead_code)]
fn demo_alignment_options() {
    println!("--- Alignment Options ---\n");

    let box_content = "Content";

    // Vertical alignments
    println!("Vertical alignment in 40x5 box:");

    let top = place(40, 5, Position::Center, Position::Top, box_content);
    let middle = place(40, 5, Position::Center, Position::Center, box_content);
    let bottom = place(40, 5, Position::Center, Position::Bottom, box_content);

    let top_box = Style::new().border(BorderStyle::Single).render(&top);
    let mid_box = Style::new().border(BorderStyle::Single).render(&middle);
    let bot_box = Style::new().border(BorderStyle::Single).render(&bottom);

    let aligned = join_horizontal(&[&top_box, &mid_box, &bot_box]);
    println!("{}\n", aligned);

    // Horizontal alignments
    println!("Horizontal alignment in 30 chars:");

    let left = place_horizontal(30, Position::LEFT, box_content);
    let center = place_horizontal(30, Position::Center, box_content);
    let right = place_horizontal(30, Position::RIGHT, box_content);

    println!("|{}|", left);
    println!("|{}|", center);
    println!("|{}|", right);
    println!();

    // Using place_vertical
    println!("Vertical placement:");
    let v_top = place_vertical(5, Position::Top, "^");
    let v_mid = place_vertical(5, Position::Center, "●");
    let v_bot = place_vertical(5, Position::Bottom, "v");

    let v_aligned = join_horizontal(&[&v_top, &v_mid, &v_bot]);
    println!("{}\n", v_aligned);
}
