//! Form demo showcasing all field types.
//!
//! Run with: cargo run --example form_demo
//!
//! This example demonstrates:
//! - All field types (Input, Select, MultiSelect, Confirm, Note, FilePicker)
//! - Form results extraction

use teapot::forms::{Field, Form, FormLayout, Group};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Teapot Form Demo ===\n");

    // Demo 1: Basic form with all field types
    demo_all_field_types()?;

    Ok(())
}

/// Helper to convert string slices to Vec<String> for bon builder options.
fn options<const N: usize>(items: [&str; N]) -> Vec<String> {
    items.into_iter().map(String::from).collect()
}

fn demo_all_field_types() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- All Field Types Demo ---\n");

    // Create a comprehensive form
    let mut form = Form::new()
        .title("Complete Registration Form")
        .layout(FormLayout::Default) // Wizard-style, one group at a time
        .group(
            Group::new()
                .title("Personal Information")
                .description("Tell us about yourself")
                .field(
                    Field::input()
                        .key("name")
                        .title("Full Name")
                        .placeholder("John Doe")
                        .description("Your legal name as it appears on documents")
                        .required(true)
                        .build(),
                )
                .field(
                    Field::input()
                        .key("email")
                        .title("Email Address")
                        .placeholder("john@example.com")
                        .description("We'll send confirmation here")
                        .required(true)
                        .build(),
                )
                .field(
                    Field::input()
                        .key("phone")
                        .title("Phone Number")
                        .placeholder("+1 (555) 123-4567")
                        .description("Optional - for account recovery")
                        .build(),
                ),
        )
        .group(
            Group::new()
                .title("Preferences")
                .description("Customize your experience")
                .field(
                    Field::select()
                        .key("theme")
                        .title("Color Theme")
                        .description("Choose your preferred appearance")
                        .options(options(["Light", "Dark", "System Default", "High Contrast"]))
                        .build(),
                )
                .field(
                    Field::select()
                        .key("language")
                        .title("Language")
                        .options(options(["English", "Spanish", "French", "German", "Japanese"]))
                        .build(),
                )
                .field(
                    Field::multi_select()
                        .key("notifications")
                        .title("Notification Preferences")
                        .description("Select all that apply (1-3 options)")
                        .options(options([
                            "Email updates",
                            "SMS alerts",
                            "Push notifications",
                            "Weekly digest",
                            "Marketing emails",
                        ]))
                        .min(1)
                        .max(3)
                        .build(),
                ),
        )
        .group(
            Group::new()
                .title("Configuration")
                .field(
                    Field::file_picker()
                        .key("config_file")
                        .title("Configuration File")
                        .description("Select your config file (optional)")
                        .extensions(options(["toml", "yaml", "json", "ini"]))
                        .build(),
                )
                .field(
                    Field::note()
                        .content(
                            "Your configuration will be validated after selection.\n\
                             Supported formats: TOML, YAML, JSON, INI",
                        )
                        .title("Note")
                        .build(),
                ),
        )
        .group(
            Group::new()
                .title("Confirmation")
                .field(
                    Field::confirm()
                        .key("terms")
                        .title("I agree to the Terms of Service")
                        .description("You must accept to continue")
                        .default(false)
                        .build(),
                )
                .field(
                    Field::confirm()
                        .key("newsletter")
                        .title("Subscribe to newsletter")
                        .description("Get weekly updates and tips")
                        .default(true)
                        .build(),
                ),
        );

    // Run the form in accessible mode (line-based prompts)
    match form.run_accessible()? {
        Some(results) => {
            println!("\n=== Form Results ===\n");
            println!("Name: {}", results.get_string("name").unwrap_or("(not set)"));
            println!("Email: {}", results.get_string("email").unwrap_or("(not set)"));
            println!("Phone: {}", results.get_string("phone").unwrap_or("(not set)"));
            println!("Theme: {}", results.get_string("theme").unwrap_or("(not set)"));
            println!("Language: {}", results.get_string("language").unwrap_or("(not set)"));

            if let Some(notifications) = results.get("notifications") {
                println!("Notifications: {:?}", notifications);
            }

            if let Some(config) = results.get("config_file") {
                println!("Config file: {:?}", config);
            }

            println!("Accepted terms: {}", results.get_bool("terms").unwrap_or(false));
            println!("Newsletter: {}", results.get_bool("newsletter").unwrap_or(false));
        },
        None => {
            println!("Form was cancelled.");
        },
    }

    Ok(())
}
