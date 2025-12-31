//! Form demo showcasing all field types and layouts.
//!
//! Run with: cargo run --example form_demo
//!
//! This example demonstrates:
//! - All field types (Input, Select, MultiSelect, Confirm, Note, FilePicker)
//! - Form layouts (Default, Stack, Columns)
//! - Dynamic content (title_fn, description_fn)
//! - Form results extraction

use teapot::forms::{
    ConfirmField, FilePickerField, Form, FormLayout, Group, InputField, MultiSelectField,
    NoteField, SelectField,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Teapot Form Demo ===\n");

    // Demo 1: Basic form with all field types
    demo_all_field_types()?;

    Ok(())
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
                    InputField::new("name")
                        .title("Full Name")
                        .placeholder("John Doe")
                        .description("Your legal name as it appears on documents")
                        .required()
                        .build(),
                )
                .field(
                    InputField::new("email")
                        .title("Email Address")
                        .placeholder("john@example.com")
                        .description("We'll send confirmation here")
                        .required()
                        .build(),
                )
                .field(
                    InputField::new("phone")
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
                    SelectField::new("theme")
                        .title("Color Theme")
                        .description("Choose your preferred appearance")
                        .options(["Light", "Dark", "System Default", "High Contrast"])
                        .build(),
                )
                .field(
                    SelectField::new("language")
                        .title("Language")
                        .options(["English", "Spanish", "French", "German", "Japanese"])
                        .build(),
                )
                .field(
                    MultiSelectField::new("notifications")
                        .title("Notification Preferences")
                        .description("Select all that apply (1-3 options)")
                        .options([
                            "Email updates",
                            "SMS alerts",
                            "Push notifications",
                            "Weekly digest",
                            "Marketing emails",
                        ])
                        .min(1)
                        .max(3)
                        .build(),
                ),
        )
        .group(
            Group::new()
                .title("Configuration")
                .field(
                    FilePickerField::new("config_file")
                        .title("Configuration File")
                        .description("Select your config file (optional)")
                        .extensions(["toml", "yaml", "json", "ini"])
                        .build(),
                )
                .field(
                    NoteField::new(
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
                    ConfirmField::new("terms")
                        .title("I agree to the Terms of Service")
                        .description("You must accept to continue")
                        .default(false)
                        .build(),
                )
                .field(
                    ConfirmField::new("newsletter")
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

#[allow(dead_code)]
fn demo_stacked_layout() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Stacked Layout Demo ---\n");

    let mut form = Form::new()
        .title("Quick Survey")
        .layout(FormLayout::Stack) // All groups visible at once
        .group(
            Group::new().title("Rating").field(
                SelectField::new("rating")
                    .title("How would you rate our service?")
                    .options(["Excellent", "Good", "Average", "Poor"])
                    .build(),
            ),
        )
        .group(
            Group::new().title("Feedback").field(
                InputField::new("feedback")
                    .title("Additional comments")
                    .placeholder("Your feedback here...")
                    .build(),
            ),
        );

    match form.run_accessible()? {
        Some(results) => {
            println!("Rating: {}", results.get_string("rating").unwrap_or(""));
            println!("Feedback: {}", results.get_string("feedback").unwrap_or(""));
        },
        None => println!("Cancelled"),
    }

    Ok(())
}

#[allow(dead_code)]
fn demo_columns_layout() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Columns Layout Demo ---\n");

    let mut form = Form::new()
        .title("Side-by-Side Form")
        .layout(FormLayout::Columns(2)) // Two columns
        .group(
            Group::new()
                .title("Left Column")
                .field(InputField::new("first_name").title("First Name").build())
                .field(InputField::new("city").title("City").build()),
        )
        .group(
            Group::new()
                .title("Right Column")
                .field(InputField::new("last_name").title("Last Name").build())
                .field(InputField::new("country").title("Country").build()),
        );

    form.run_accessible()?;
    Ok(())
}

#[allow(dead_code)]
fn demo_dynamic_content() -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    println!("--- Dynamic Content Demo ---\n");

    let attempt_count = Arc::new(AtomicUsize::new(1));
    let attempt_clone = attempt_count.clone();

    let mut form = Form::new().title("Login").group(
        Group::new()
            .field(
                InputField::new("password")
                    .title_fn(move || {
                        format!("Password (attempt {})", attempt_clone.load(Ordering::SeqCst))
                    })
                    .description_fn(|| "Must be at least 8 characters".to_string())
                    .build(),
            )
            .field(ConfirmField::new("remember").title("Remember me").build()),
    );

    form.run_accessible()?;
    Ok(())
}
