use std::fs;
use crate::predictor::candidate::{Candidate, CandidateSource};
use crate::predictor::context::{PredictionContext, ProjectType};
use crate::providers::CandidateProvider;

pub struct ProjectProvider;

impl Default for ProjectProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectProvider {
    pub fn new() -> Self {
        Self
    }

    fn extract_package_json_scripts(context: &PredictionContext) -> Vec<(String, String)> {
        let pkg_path = context.cwd.join("package.json");
        if let Ok(content) = fs::read_to_string(pkg_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(scripts) = val.get("scripts").and_then(|s| s.as_object()) {
                    return scripts
                        .iter()
                        .map(|(k, v)| {
                            let script_cmd = v.as_str().unwrap_or("").to_string();
                            (format!("npm run {}", k), format!("Script: {}", script_cmd))
                        })
                        .collect();
                }
            }
        }
        vec![
            ("npm run dev".to_string(), "Run development server".to_string()),
            ("npm run build".to_string(), "Build for production".to_string()),
            ("npm test".to_string(), "Run test suite".to_string()),
            ("npm install".to_string(), "Install dependencies".to_string()),
        ]
    }
}

impl CandidateProvider for ProjectProvider {
    fn name(&self) -> &'static str {
        "Project"
    }

    fn suggest(&self, context: &PredictionContext) -> Vec<Candidate> {
        let Some(project_type) = context.project_type else {
            return Vec::new();
        };

        let input = context.input_up_to_cursor().trim();
        let mut candidates = Vec::new();

        let suggestions: Vec<(String, String)> = match project_type {
            ProjectType::Rust => vec![
                ("cargo build".to_string(), "Compile the current package".to_string()),
                ("cargo run".to_string(), "Run the main binary".to_string()),
                ("cargo test".to_string(), "Execute all unit and integration tests".to_string()),
                ("cargo check".to_string(), "Analyze package and check for errors".to_string()),
                ("cargo clippy".to_string(), "Run linter checks".to_string()),
                ("cargo fmt".to_string(), "Format code with rustfmt".to_string()),
                ("cargo doc --open".to_string(), "Build and view package documentation".to_string()),
            ],
            ProjectType::Node => Self::extract_package_json_scripts(context),
            ProjectType::Python => vec![
                ("pytest".to_string(), "Run Python test suite".to_string()),
                ("python -m unittest".to_string(), "Run standard unit tests".to_string()),
                ("pip install -r requirements.txt".to_string(), "Install requirements".to_string()),
                ("uv run".to_string(), "Run with uv virtualenv".to_string()),
                ("poetry run".to_string(), "Run with Poetry environment".to_string()),
            ],
            ProjectType::Go => vec![
                ("go run .".to_string(), "Compile and run Go package".to_string()),
                ("go test ./...".to_string(), "Run all Go tests".to_string()),
                ("go build .".to_string(), "Compile Go binaries".to_string()),
                ("go mod tidy".to_string(), "Add missing and remove unused modules".to_string()),
            ],
            ProjectType::CMake => vec![
                ("cmake -B build".to_string(), "Configure CMake build tree".to_string()),
                ("cmake --build build".to_string(), "Build CMake target binaries".to_string()),
                ("ctest --test-dir build".to_string(), "Run CMake test suite".to_string()),
            ],
            ProjectType::Cpp => vec![
                ("make".to_string(), "Build default Makefile target".to_string()),
                ("make test".to_string(), "Run Makefile test target".to_string()),
                ("make clean".to_string(), "Clean build artifacts".to_string()),
            ],
            ProjectType::Ruby => vec![
                ("bundle exec rake".to_string(), "Run default rake task".to_string()),
                ("bundle install".to_string(), "Install ruby gems".to_string()),
            ],
            ProjectType::Java => vec![
                ("./gradlew build".to_string(), "Assemble and test Gradle project".to_string()),
                ("mvn clean package".to_string(), "Clean and package Maven project".to_string()),
            ],
        };

        for (cmd, desc) in suggestions {
            if input.is_empty() || cmd.starts_with(input) {
                candidates.push(
                    Candidate::new(cmd, CandidateSource::Project, 85.0)
                        .with_description(desc)
                        .with_prefix_len(input.len()),
                );
            }
        }

        candidates
    }
}
