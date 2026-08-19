use crate::predictor::candidate::{Candidate, CandidateSource};
use crate::predictor::context::PredictionContext;
use crate::providers::CandidateProvider;

pub struct OptionProvider;

impl Default for OptionProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl OptionProvider {
    pub fn new() -> Self {
        Self
    }
}

impl CandidateProvider for OptionProvider {
    fn name(&self) -> &'static str {
        "Option"
    }

    fn suggest(&self, context: &PredictionContext) -> Vec<Candidate> {
        let token = context.current_token();
        if !token.starts_with('-') {
            return Vec::new();
        }

        let cmd = context.command_name().unwrap_or("");
        let mut candidates = Vec::new();

        let options: &[(&str, &str)] = match cmd {
            "ls" => &[
                ("-la", "List all files including hidden in long format"),
                ("-lh", "List long format with human readable sizes"),
                ("-a", "Do not ignore entries starting with ."),
                ("-l", "Use a long listing format"),
                ("-t", "Sort by time, newest first"),
                ("-R", "List subdirectories recursively"),
                ("-1", "List one file per line"),
            ],
            "grep" => &[
                ("-r", "Search directories recursively"),
                ("-i", "Ignore case distinctions in patterns and data"),
                ("-n", "Prefix each line of output with line number"),
                ("-E", "Interpret PATTERNS as extended regular expressions"),
                ("-v", "Invert the sense of matching"),
                ("-w", "Select only those lines containing whole words"),
                ("-l", "Suppress normal output; print only file names"),
            ],
            "tar" => &[
                ("-czvf", "Create gzip archive with verbose progress"),
                ("-xzvf", "Extract gzip archive with verbose progress"),
                ("-tf", "List archive contents"),
                ("-cjvf", "Create bzip2 archive"),
                ("-xjvf", "Extract bzip2 archive"),
            ],
            "cargo" => &[
                ("--release", "Build or run in release mode with optimizations"),
                ("--workspace", "Build or test all members in workspace"),
                ("--all-targets", "Check or test all targets (lib, bins, tests, examples)"),
                ("--features", "Space or comma separated list of features to activate"),
                ("--no-default-features", "Do not activate the default feature"),
                ("--verbose", "Use verbose output"),
            ],
            "docker" => &[
                ("-it", "Allocate pseudo-TTY connected to container stdin"),
                ("-d", "Run container in background and print container ID"),
                ("--rm", "Automatically remove container when it exits"),
                ("-p", "Publish a container port to the host"),
                ("-v", "Bind mount a volume"),
                ("--name", "Assign a name to the container"),
            ],
            "find" => &[
                ("-name", "Base of file name matches shell pattern"),
                ("-type f", "Filter regular files"),
                ("-type d", "Filter directories"),
                ("-mtime -7", "Files modified within the last 7 days"),
                ("-exec", "Execute command for each matched file"),
            ],
            _ => &[
                ("--help", "Display help message and exit"),
                ("--version", "Output version information and exit"),
                ("-v", "Verbose output"),
                ("-f", "Force execution"),
            ],
        };

        for (opt, desc) in options {
            if opt.starts_with(token) {
                candidates.push(
                    Candidate::new(*opt, CandidateSource::Option, 70.0)
                        .with_description(*desc)
                        .with_prefix_len(token.len()),
                );
            }
        }

        candidates
    }
}
