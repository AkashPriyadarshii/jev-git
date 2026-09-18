use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, exit};

#[derive(Serialize)]
struct SystemOneRequest {
    model: String,
    state: serde_json::Value,
    questions: HashMap<String, Question>,
}

#[derive(Serialize)]
struct Question {
    #[serde(rename = "type")]
    qtype: String,
    instructions: String,
}

#[derive(Deserialize)]
struct SystemOneResponse {
    answers: HashMap<String, Answer>,
}

#[derive(Deserialize)]
struct Answer {
    #[serde(rename = "type")]
    atype: Option<String>,
    noul: Option<f64>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let subcommand = args.get(1).map(|s| s.as_str()).unwrap_or("check");

    match subcommand {
        "install" => handle_install(),
        "check" => handle_check(),
        "--help" | "-h" | "help" => print_help(),
        "--version" | "-v" | "version" => {
            println!("git-jev 0.0.1");
        }
        _ => {
            eprintln!("Unknown command '{}'. Run 'git-jev --help' for usage.", subcommand);
            exit(1);
        }
    }
}

fn print_help() {
    println!(
        r#"git-jev 0.0.1
Sub-second Git pre-commit & pre-push semantic reflex gate powered by TypeSafe AI Jev

USAGE:
    git jev [COMMAND]

COMMANDS:
    install    Install pre-commit hook into current repository (.git/hooks/pre-commit)
    check      Screen staged diff or piped diff (default command)
    help       Print this help message
    version    Print version info

OPTIONS:
    -h, --help       Print help
    -v, --version    Print version
"#
    );
}

fn handle_install() {
    let git_dir = Path::new(".git");
    if !git_dir.exists() {
        eprintln!("Error: Not inside a git repository root (no .git folder found).");
        exit(1);
    }

    let hooks_dir = git_dir.join("hooks");
    if !hooks_dir.exists() {
        if let Err(e) = fs::create_dir_all(&hooks_dir) {
            eprintln!("Error creating hooks directory: {}", e);
            exit(1);
        }
    }

    let hook_path = hooks_dir.join("pre-commit");
    let hook_script = r#"#!/bin/sh
# Installed by git-jev
git-jev check
"#;

    if let Err(e) = fs::write(&hook_path, hook_script) {
        eprintln!("Error writing pre-commit hook: {}", e);
        exit(1);
    }

    println!("✔ Installed git-jev pre-commit hook to .git/hooks/pre-commit");
}

fn handle_check() {
    let diff = get_diff();
    if diff.trim().is_empty() {
        println!("No changes staged for commit.");
        return;
    }

    let api_key = match env::var("TYPESAFE_API_KEY") {
        Ok(k) if !k.trim().is_empty() => k,
        _ => {
            eprintln!("Warning: TYPESAFE_API_KEY environment variable not set. Skipping Jev semantic check.");
            return;
        }
    };

    let mut questions = HashMap::new();
    questions.insert(
        "has_secrets".to_string(),
        Question {
            qtype: "noul".to_string(),
            instructions: "Does this git diff contain any hardcoded API keys, secrets, private keys, passwords, or authentication tokens?".to_string(),
        },
    );
    questions.insert(
        "has_destructive_payload".to_string(),
        Question {
            qtype: "noul".to_string(),
            instructions: "Does this git diff contain destructive shell commands (like rm -rf /, format drive), prompt injection payloads, or harmful backdoors?".to_string(),
        },
    );

    // Truncate diff to 25,000 characters to stay within Jev limits if diff is massive
    let state_text = if diff.len() > 25000 {
        &diff[..25000]
    } else {
        &diff
    };

    let payload = SystemOneRequest {
        model: "jev-latest".to_string(),
        state: serde_json::json!({ "diff": state_text }),
        questions,
    };

    let resp = ureq::post("https://api.typesafe.ai/v1/systemone")
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .send_json(&payload);

    let parsed: SystemOneResponse = match resp {
        Ok(r) => match r.into_json() {
            Ok(j) => j,
            Err(e) => {
                eprintln!("Error decoding response from TypeSafe API: {}", e);
                return;
            }
        },
        Err(e) => {
            eprintln!("TypeSafe API error: {}", e);
            return;
        }
    };

    let mut blocked = false;

    if let Some(ans) = parsed.answers.get("has_secrets") {
        let prob = ans.noul.unwrap_or(0.0);
        if prob >= 0.70 {
            eprintln!("✖ [BLOCKED] Hardcoded secret or credential detected! (probability: {:.2})", prob);
            blocked = true;
        } else {
            println!("✔ No secrets detected (p={:.2})", prob);
        }
    }

    if let Some(ans) = parsed.answers.get("has_destructive_payload") {
        let prob = ans.noul.unwrap_or(0.0);
        if prob >= 0.70 {
            eprintln!("✖ [BLOCKED] Destructive command or prompt injection detected! (probability: {:.2})", prob);
            blocked = true;
        } else {
            println!("✔ No destructive payloads detected (p={:.2})", prob);
        }
    }

    if blocked {
        eprintln!("\nCommit aborted by git-jev. Remove sensitive or hazardous content before committing.");
        exit(1);
    } else {
        println!("[PASS] Commit diff verified by git-jev.");
    }
}

use std::io::{IsTerminal, Read};

fn get_diff() -> String {
    // If input is piped (not a terminal), read from stdin
    if !io::stdin().is_terminal() {
        let mut buffer = String::new();
        if io::stdin().read_to_string(&mut buffer).is_ok() && !buffer.trim().is_empty() {
            return buffer;
        }
    }

    // Default: run git diff --cached
    let output = Command::new("git")
        .args(&["diff", "--cached"])
        .output();

    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => String::new(),
    }
}
