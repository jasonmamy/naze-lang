use std::fs;
use std::path::Path;

const NAZE_TOML_TEMPLATE: &str = r#"[app]
name = "{{name}}"
version = "0.1.0"

[build]
entry = "app.naze"
output = "dist/"
"#;

const APP_NAZE_TEMPLATE: &str = r#"-- {{name}}
-- Created with nazec

app "{{name}}" {
  state count = 0
  state items = ["Naze", "is", "declarative"]

  column padding: 20px, gap: 16px {
    heading "Welcome to {{name}}!"
    text "Count: {count}"

    row gap: 12px {
      rect width: 120px, height: 50px, color: #2563eb, radius: 8px {
        text "Increment"
        on click: set count = count + 1
      }
      rect width: 120px, height: 50px, color: #dc2626, radius: 8px {
        text "Reset"
        on click: set count = 0
      }
    }

    if count > 0 {
      text "You clicked {count} times!"
    } else {
      text "Click Increment to get started."
    }

    heading "Items:"
    each item in items {
      text "- {item}"
    }

    text "Edit app.naze and run nazec build to see changes."
  }
}
"#;

pub fn run(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        return Err(format!("directory '{}' already exists", name).into());
    }

    // Create project directory structure
    fs::create_dir_all(project_dir.join("components"))?;

    // Write naze.toml
    let toml_content = NAZE_TOML_TEMPLATE.replace("{{name}}", name);
    fs::write(project_dir.join("naze.toml"), toml_content)?;

    // Write app.naze
    let app_content = APP_NAZE_TEMPLATE.replace("{{name}}", name);
    fs::write(project_dir.join("app.naze"), app_content)?;

    eprintln!("created project '{name}'");
    eprintln!();
    eprintln!("  cd {name}");
    eprintln!("  nazec build");

    Ok(())
}
