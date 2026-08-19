use crate::predictor::candidate::{Candidate, CandidateSource};
use crate::predictor::context::PredictionContext;
use crate::providers::CandidateProvider;

pub struct GitProvider;

impl Default for GitProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GitProvider {
    pub fn new() -> Self {
        Self
    }

    fn subcommands() -> &'static [(&'static str, &'static str, f32)] {
        &[
            ("status", "Show the working tree status", 90.0),
            ("add", "Add file contents to the staging area", 88.0),
            ("commit", "Record changes to the repository", 87.0),
            ("push", "Update remote refs along with associated objects", 86.0),
            ("pull", "Fetch from and integrate with another repository", 85.0),
            ("checkout", "Switch branches or restore working tree files", 84.0),
            ("switch", "Switch branches", 83.0),
            ("branch", "List, create, or delete branches", 82.0),
            ("diff", "Show changes between commits, commit and working tree", 81.0),
            ("log", "Show commit logs", 80.0),
            ("merge", "Join two or more development histories together", 78.0),
            ("rebase", "Reapply commits on top of another base tip", 77.0),
            ("stash", "Stash the changes in a dirty working directory away", 76.0),
            ("fetch", "Download objects and refs from another repository", 75.0),
            ("clone", "Clone a repository into a new directory", 74.0),
            ("reset", "Reset current HEAD to the specified state", 73.0),
            ("restore", "Restore working tree files", 72.0),
            ("remote", "Manage set of tracked repositories", 71.0),
            ("tag", "Create, list, delete or verify a tag object", 70.0),
        ]
    }
}

impl CandidateProvider for GitProvider {
    fn name(&self) -> &'static str {
        "Git"
    }

    fn suggest(&self, context: &PredictionContext) -> Vec<Candidate> {
        let input = context.input_up_to_cursor();
        let tokens: Vec<&str> = input.split_whitespace().collect();

        if tokens.is_empty() || tokens[0] != "git" {
            return Vec::new();
        }

        let mut candidates = Vec::new();
        let is_trailing_space = input.ends_with(' ');

        // Case 1: Typing `git` or `git <subcmd_prefix>`
        if (tokens.len() == 1 && is_trailing_space) || (tokens.len() == 2 && !is_trailing_space) {
            let prefix = if tokens.len() == 2 { tokens[1] } else { "" };
            for (subcmd, desc, score) in Self::subcommands() {
                if prefix.is_empty() || subcmd.starts_with(prefix) {
                    candidates.push(
                        Candidate::new(*subcmd, CandidateSource::Git, *score)
                            .with_description(*desc)
                            .with_prefix_len(prefix.len()),
                    );
                }
            }
            return candidates;
        }

        // Case 2: Subcommand is known, suggesting arguments/branches
        if tokens.len() >= 2 {
            let subcmd = tokens[1];
            let last_token = if is_trailing_space { "" } else { tokens.last().copied().unwrap_or("") };

            match subcmd {
                "checkout" | "switch" | "merge" | "rebase" => {
                    if let Some(git) = &context.git {
                        for branch in &git.branches {
                            if last_token.is_empty() || branch.starts_with(last_token) {
                                candidates.push(
                                    Candidate::new(branch.clone(), CandidateSource::Git, 80.0)
                                        .with_description("Branch")
                                        .with_prefix_len(last_token.len()),
                                );
                            }
                        }
                    }
                }
                "push" => {
                    if let Some(git) = &context.git {
                        let push_current = format!("origin {}", git.branch);
                        let push_upstream = format!("--set-upstream origin {}", git.branch);
                        if last_token.is_empty() || push_current.starts_with(last_token) {
                            candidates.push(
                                Candidate::new(push_current, CandidateSource::Git, 85.0)
                                    .with_description("Push current branch to origin")
                                    .with_prefix_len(last_token.len()),
                            );
                        }
                        if last_token.is_empty() || push_upstream.starts_with(last_token) {
                            candidates.push(
                                Candidate::new(push_upstream, CandidateSource::Git, 80.0)
                                    .with_description("Push and set upstream tracking")
                                    .with_prefix_len(last_token.len()),
                            );
                        }
                    }
                }
                "commit" => {
                    let commit_m = "-m \"\"";
                    let commit_am = "-am \"\"";
                    if last_token.is_empty() || commit_m.starts_with(last_token) {
                        candidates.push(
                            Candidate::new(commit_m, CandidateSource::Git, 80.0)
                                .with_description("Commit with message")
                                .with_prefix_len(last_token.len()),
                        );
                    }
                    if last_token.is_empty() || commit_am.starts_with(last_token) {
                        candidates.push(
                            Candidate::new(commit_am, CandidateSource::Git, 75.0)
                                .with_description("Stage all tracked and commit with message")
                                .with_prefix_len(last_token.len()),
                        );
                    }
                }
                "add" => {
                    if last_token.is_empty() || ".".starts_with(last_token) {
                        candidates.push(
                            Candidate::new(".", CandidateSource::Git, 85.0)
                                .with_description("Stage all changes in current directory")
                                .with_prefix_len(last_token.len()),
                        );
                        candidates.push(
                            Candidate::new("-A", CandidateSource::Git, 80.0)
                                .with_description("Stage all tracked and untracked changes")
                                .with_prefix_len(last_token.len()),
                        );
                    }
                }
                "stash" => {
                    for op in &["pop", "apply", "list", "drop", "save"] {
                        if last_token.is_empty() || op.starts_with(last_token) {
                            candidates.push(
                                Candidate::new(*op, CandidateSource::Git, 80.0)
                                    .with_description(format!("Git stash {}", op))
                                    .with_prefix_len(last_token.len()),
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        candidates
    }
}
