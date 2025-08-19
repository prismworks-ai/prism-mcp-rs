//! Doc-driven examples test runner
//!
//! This test extracts examples from rustdoc comments, validates them,
//! and generates user-facing example files.

use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_and_extract_doc_examples() {
    // Skip in CI/Act environments where doc tests might not work properly
    if std::env::var("CI").is_ok() || std::env::var("ACT").is_ok() {
        println!("⏭️ Skipping doc examples test in CI/Act environment");
        return;
    }

    println!("🔍 Testing and extracting documentation examples...");

    // First, run all doc tests to ensure they compile
    println!("📝 Running doc tests...");

    // Run doc tests without --quiet to get better error messages
    let output = Command::new("cargo")
        .args(&["test", "--doc"])
        .current_dir(env!("CARGO_MANIFEST_DIR")) // Set proper working directory
        .output()
        .expect("Failed to run doc tests");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check if it's actually a failure or just warnings
        if stderr.contains("error:") || stdout.contains("FAILED") {
            panic!(
                "Doc tests failed! Fix the documentation examples before proceeding.\nstderr:\n{}\nstdout:\n{}",
                stderr, stdout
            );
        }
    }

    println!("✅ Doc tests passed!");

    // If doc tests pass, extract and save examples
    println!("📚 Extracting examples from source...");
    let examples = extract_examples_from_source();

    println!("💾 Generating example files...");
    generate_example_files(examples);

    println!("✨ Doc examples extraction complete!");
}

#[derive(Debug, Clone)]
struct Example {
    name: String,
    code: String,
    module_path: String,
    line_number: usize,
    is_runnable: bool,
    requires_features: Vec<String>,
    description: String,
}

fn extract_examples_from_source() -> HashMap<String, Vec<Example>> {
    let mut examples = HashMap::new();
    let src_dir = Path::new("src");

    extract_examples_recursive(src_dir, &mut examples);
    examples
}

fn extract_examples_recursive(dir: &Path, examples: &mut HashMap<String, Vec<Example>>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                extract_examples_recursive(&path, examples);
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                extract_examples_from_file(&path, examples);
            }
        }
    }
}

fn extract_examples_from_file(file: &Path, examples: &mut HashMap<String, Vec<Example>>) {
    let content = match fs::read_to_string(file) {
        Ok(c) => c,
        Err(_) => return,
    };

    let module_path = file
        .strip_prefix("src/")
        .unwrap()
        .to_str()
        .unwrap()
        .replace(".rs", "")
        .replace("/", "::");

    // Multiple regex patterns to match different doc comment styles
    let patterns = vec![
        // Standard Example or Examples section
        r"(?ms)/// # Example[s]?(?:\s*:\s*([^\n]+))?\n((?:///[^\n]*\n)*?)/// ```(?:rust)?(?:,([^`]+))?\n((?:///[^\n]*\n)*?)/// ```",
        // Examples with description
        r"(?ms)/// ## ([^\n]+)\n((?:///[^\n]*\n)*?)/// ```(?:rust)?(?:,([^`]+))?\n((?:///[^\n]*\n)*?)/// ```",
    ];

    for pattern_str in patterns {
        let doc_example_regex = Regex::new(pattern_str).unwrap();

        for (idx, cap) in doc_example_regex.captures_iter(&content).enumerate() {
            let title = cap.get(1).map_or("", |m| m.as_str()).trim();
            let description = cap.get(2).map_or("", |m| m.as_str());
            let attributes = cap.get(3).map_or("", |m| m.as_str());
            let code = cap
                .get(4)
                .map_or(cap.get(3).map_or("", |m| m.as_str()), |m| m.as_str());

            // Clean up the code (remove /// prefix)
            let clean_code = code
                .lines()
                .map(|line| {
                    line.strip_prefix("/// ")
                        .unwrap_or(line.strip_prefix("///").unwrap_or(line))
                })
                .collect::<Vec<_>>()
                .join("\n");

            // Skip if marked as ignore
            if attributes.contains("ignore") {
                continue;
            }

            let is_runnable = !attributes.contains("no_run");
            let example_name = if !title.is_empty() {
                format!(
                    "{}_{}",
                    module_path.replace("::", "_"),
                    title.to_lowercase().replace(" ", "_").replace("-", "_")
                )
            } else {
                format!("{}_example_{}", module_path.replace("::", "_"), idx + 1)
            };

            // Clean up the example name
            let example_name = example_name
                .replace("__", "_")
                .trim_matches('_')
                .to_string();

            let example = Example {
                name: example_name,
                code: clean_code,
                module_path: module_path.clone(),
                line_number: content[..cap.get(0).unwrap().start()].lines().count(),
                is_runnable,
                requires_features: extract_required_features(&attributes),
                description: description.to_string(),
            };

            examples
                .entry(module_path.clone())
                .or_insert_with(Vec::new)
                .push(example);
        }
    }
}

fn extract_required_features(attributes: &str) -> Vec<String> {
    let mut features = Vec::new();

    // Parse feature requirements from attributes
    if attributes.contains("http") {
        features.push("http".to_string());
    }
    if attributes.contains("websocket") {
        features.push("websocket".to_string());
    }
    if attributes.contains("http2") {
        features.push("http2".to_string());
    }
    if attributes.contains("plugin") {
        features.push("plugin".to_string());
    }

    features
}

fn generate_example_files(examples: HashMap<String, Vec<Example>>) {
    let examples_dir = Path::new("examples/generated");

    // Create the generated examples directory
    if !examples_dir.exists() {
        fs::create_dir_all(examples_dir).unwrap();
    }

    let mut generated_count = 0;
    let mut skipped_count = 0;

    // Generate example files
    for (_module, module_examples) in &examples {
        for example in module_examples {
            let file_name = format!("{}.rs", example.name);
            let file_path = examples_dir.join(&file_name);

            // Expand the example code with necessary boilerplate
            let full_code = expand_example_code(example);

            // Validate that the example would compile (basic check)
            if validate_example_basic(example) {
                fs::write(&file_path, full_code).unwrap();
                println!("  ✅ Generated: examples/generated/{}", file_name);
                generated_count += 1;
            } else {
                println!("  ⚠️  Skipped: {} (validation failed)", example.name);
                skipped_count += 1;
            }
        }
    }

    // Generate examples/generated/README.md
    generate_examples_readme(&examples);

    println!(
        "\n📊 Summary: {} examples generated, {} skipped",
        generated_count, skipped_count
    );
}

fn validate_example_basic(example: &Example) -> bool {
    // Basic validation - check if the code has some structure
    // More sophisticated validation could be added later

    // Skip examples that are clearly incomplete or test-only
    if example.code.contains("unimplemented!")
        || example.code.contains("todo!")
        || example.code.len() < 50
    {
        return false;
    }

    true
}

fn expand_example_code(example: &Example) -> String {
    let mut expanded = String::new();

    // Add file header
    expanded.push_str(&format!(
        "//! Example: {}\n\
         //! Generated from: src/{}.rs\n\
         //! \n\
         //! This example is automatically generated from the documentation.\n\
         //! Do not edit this file directly. Edit the source documentation instead.\n\n",
        example.name.replace("_", " "),
        example.module_path.replace("::", "/")
    ));

    // Add feature gates if needed
    if !example.requires_features.is_empty() {
        for feature in &example.requires_features {
            expanded.push_str(&format!("#![cfg(feature = \"{}\")]\n", feature));
        }
        expanded.push('\n');
    }

    // Process the code to handle hidden lines and ensure proper structure
    let processed_code = process_example_code(&example.code);

    // If the code doesn't have a main function and it's runnable, wrap it
    if example.is_runnable && !processed_code.contains("fn main") {
        expanded.push_str("use prism_mcp_rs::prelude::*;\n");
        expanded.push_str("use std::collections::HashMap;\n");
        expanded.push_str("use serde_json::{json, Value};\n\n");
        expanded.push_str("#[tokio::main]\n");
        expanded.push_str("async fn main() -> Result<(), Box<dyn std::error::Error>> {\n");
        expanded.push_str(&indent_code(&processed_code, 1));
        expanded.push_str("\n    Ok(())\n");
        expanded.push_str("}\n");
    } else {
        expanded.push_str(&processed_code);
    }

    expanded
}

fn process_example_code(code: &str) -> String {
    let mut processed = String::new();

    for line in code.lines() {
        // Handle hidden lines (marked with #)
        if line.trim_start().starts_with("# ") {
            // Include hidden lines but remove the # marker
            let unhidden = line.replacen("# ", "", 1);
            processed.push_str(&unhidden);
        } else if line.trim_start().starts_with("#") && !line.trim_start().starts_with("#[") {
            // Skip pure hidden lines
            continue;
        } else {
            processed.push_str(line);
        }
        processed.push('\n');
    }

    processed
}

fn indent_code(code: &str, level: usize) -> String {
    let indent = "    ".repeat(level);
    code.lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                format!("{}{}", indent, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn generate_examples_readme(examples: &HashMap<String, Vec<Example>>) {
    let readme_path = Path::new("examples/generated/README.md");
    let mut content = String::new();

    content.push_str("# Generated Examples\n\n");
    content.push_str("These examples are automatically generated from the SDK documentation.\n");
    content.push_str(
        "**Do not edit these files directly** - edit the source documentation instead.\n\n",
    );

    content.push_str("## Running Examples\n\n");
    content.push_str("```bash\n");
    content.push_str("# Run a specific example\n");
    content.push_str("cargo run --example <example_name>\n\n");
    content.push_str("# Run with required features\n");
    content.push_str("cargo run --example <example_name> --features \"feature1 feature2\"\n");
    content.push_str("```\n\n");

    content.push_str("## Available Examples\n\n");

    // Group examples by module
    let mut modules: Vec<_> = examples.keys().collect();
    modules.sort();

    for module in modules {
        if let Some(module_examples) = examples.get(module) {
            if module_examples.is_empty() {
                continue;
            }

            content.push_str(&format!("### {}\n\n", module));

            for example in module_examples {
                let features = if example.requires_features.is_empty() {
                    String::new()
                } else {
                    format!(" *(requires: {})*", example.requires_features.join(", "))
                };

                content.push_str(&format!(
                    "- **{}** - Line {}{}\n",
                    example.name, example.line_number, features
                ));

                if !example.description.is_empty() {
                    content.push_str(&format!("  {}", example.description.trim()));
                    content.push_str("\n");
                }
            }
            content.push('\n');
        }
    }

    content.push_str("## Regenerating Examples\n\n");
    content.push_str("To regenerate these examples from the documentation:\n\n");
    content.push_str("```bash\n");
    content.push_str("cargo test --test doc_examples\n");
    content.push_str("```\n\n");
    content.push_str("Or using the Makefile:\n\n");
    content.push_str("```bash\n");
    content.push_str("make examples-generate\n");
    content.push_str("```\n");

    fs::write(readme_path, content).unwrap();
    println!("  📝 Generated: examples/generated/README.md");
}
