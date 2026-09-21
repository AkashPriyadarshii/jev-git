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
    criteria: HashMap<String, String>,
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
# Installed by git-jev (chained: existing hook content preserved below)
git-jev check
"#;

    if hook_path.exists() {
        let existing = fs::read_to_string(&hook_path).unwrap_or_default();
        if existing.contains("git-jev check") {
            println!("✔ git-jev already wired in .git/hooks/pre-commit");
        } else {
            let chained = format!("{}\n{}", hook_script, existing);
            if let Err(e) = fs::write(&hook_path, chained) {
                eprintln!("Error chaining pre-commit hook: {}", e);
                exit(1);
            }
            set_executable(&hook_path);
            println!("✔ Chained git-jev into existing .git/hooks/pre-commit");
        }
    } else {
        if let Err(e) = fs::write(&hook_path, hook_script) {
            eprintln!("Error writing pre-commit hook: {}", e);
            exit(1);
        }
        set_executable(&hook_path);
        println!("✔ Installed git-jev pre-commit hook to .git/hooks/pre-commit");
    }

    let push_path = hooks_dir.join("pre-push");
    let push_script = "#!/bin/sh\n# Installed by git-jev\ngit-jev check\n";
    if !push_path.exists() {
        if let Err(e) = fs::write(&push_path, push_script) {
            eprintln!("Error writing pre-push hook: {}", e);
            exit(1);
        }
        set_executable(&push_path);
        println!("✔ Installed git-jev pre-push hook to .git/hooks/pre-push");
    }
}

#[cfg(unix)]
fn set_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = fs::metadata(path) {
        let mut perm = meta.permissions();
        perm.set_mode(0o755);
        let _ = fs::set_permissions(path, perm);
    }
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) {}

fn handle_check() {
    let diff = get_diff();
    if diff.trim().is_empty() {
        println!("No changes staged for commit.");
        return;
    }

    let api_key = match env::var("TYPESAFE_API_KEY") {
        Ok(k) if !k.trim().is_empty() => k,
        _ => {
            eprintln!("BLOCKED: TYPESAFE_API_KEY not set. No check ran, commit refused.");
            exit(2);
        }
    };

    let mut questions = HashMap::new();
    let mut bool_criteria = HashMap::new();
    bool_criteria.insert("true".to_string(), "The diff clearly contains the flagged content.".to_string());
    bool_criteria.insert("false".to_string(), "The diff does not contain the flagged content.".to_string());
    questions.insert(
        "has_secrets".to_string(),
        Question {
            qtype: "noul".to_string(),
            instructions: "Does this git diff contain any hardcoded API keys, secrets, private keys, passwords, or authentication tokens?".to_string(),
            criteria: bool_criteria.clone(),
        },
    );
    questions.insert(
        "has_destructive_payload".to_string(),
        Question {
            qtype: "noul".to_string(),
            instructions: "Does this git diff contain destructive shell commands (like rm -rf /, format drive), prompt injection payloads, or harmful backdoors?".to_string(),
            criteria: bool_criteria,
        },
    );

    // Truncate diff at a char boundary; warn because secrets hide in tails.
    let state_text = if diff.len() > 25000 {
        eprintln!("Warning: diff truncated to 25,000 chars for scoring; review the tail by hand.");
        let cut = diff.floor_char_boundary(25000);
        &diff[..cut]
    } else {
        &diff
    };

    let payload = SystemOneRequest {
        model: "jev-1.13.0".to_string(),
        state: serde_json::json!({ "diff": state_text }),
        questions,
    };

    let resp = ureq::post("https://api.typesafe.ai/v1/systemone")
        .set("Authorization", &format!("Bearer {}", api_key))
        .set("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(8))
        .send_json(&payload);

    let parsed: SystemOneResponse = match resp {
        Ok(r) => match r.into_json() {
            Ok(j) => j,
            Err(e) => {
                eprintln!("BLOCKED: could not decode TypeSafe response ({}). Commit refused.", e);
                exit(2);
            }
        },
        Err(e) => {
            eprintln!("BLOCKED: TypeSafe API error ({}). Commit refused.", e);
            exit(2);
        }
    };

    let mut blocked = false;

    blocked |= gate("secret or credential", parsed.answers.get("has_secrets"));
    blocked |= gate("destructive payload", parsed.answers.get("has_destructive_payload"));

    if blocked {
        eprintln!("\nCommit aborted by git-jev. Remove sensitive or hazardous content before committing.");
        exit(1);
    } else {
        println!("[PASS] Commit diff verified by git-jev.");
    }
}

/// One threshold band per answer. Missing answer = fail closed.
fn gate(label: &str, ans: Option<&Answer>) -> bool {
    match verdict(ans.and_then(|a| a.noul)) {
        Verdict::Block => {
            eprintln!("✖ [BLOCKED] {} detected! Refusing commit.", label);
            true
        }
        Verdict::Missing => {
            eprintln!("BLOCKED: Jev answer for '{}' missing or unparsable. Commit refused.", label);
            exit(2);
        }
        Verdict::Warn(p) => {
            eprintln!("⚠ [WARN] Possible {} (p={:.2}). Proceed only after manual review.", label, p);
            false
        }
        Verdict::Pass(p) => {
            println!("✔ No {} detected (p={:.2})", label, p);
            false
        }
    }
}

#[derive(Debug, PartialEq)]
enum Verdict {
    Block,
    Warn(f64),
    Pass(f64),
    Missing,
}

fn verdict(prob: Option<f64>) -> Verdict {
    match prob {
        None => Verdict::Missing,
        Some(p) if p >= 0.80 => Verdict::Block,
        Some(p) if p >= 0.55 => Verdict::Warn(p),
        Some(p) => Verdict::Pass(p),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bands() {
        assert_eq!(verdict(None), Verdict::Missing);
        assert_eq!(verdict(Some(0.9)), Verdict::Block);
        assert_eq!(verdict(Some(0.8)), Verdict::Block);
        assert_eq!(verdict(Some(0.6)), Verdict::Warn(0.6));
        assert_eq!(verdict(Some(0.1)), Verdict::Pass(0.1));
    }

    #[test]
    fn truncates_at_char_boundary() {
        let s = "a".repeat(24999) + "é";
        let cut = s.floor_char_boundary(25000);
        assert!(s[..cut].chars().count() <= 25000);
        assert!(std::str::from_utf8(&s.as_bytes()[..cut]).is_ok());
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
