//! AlfAlfa-specific functionality: directory setup, identity config, lf skills, journal.

use std::path::{Path, PathBuf};

/// Returns the base AlfAlfa directory: ~/.alfalfa
pub fn base_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Cannot determine home directory")?;
    Ok(home.join(".alfalfa"))
}

/// Returns the XDG overrides that redirect the opencode server data into ~/.alfalfa/
pub fn xdg_env_overrides() -> Result<Vec<(&'static str, String)>, String> {
    let base = base_dir()?;
    Ok(vec![
        (
            "XDG_DATA_HOME",
            base.join("data").to_string_lossy().into_owned(),
        ),
        (
            "XDG_CONFIG_HOME",
            base.join("config").to_string_lossy().into_owned(),
        ),
        (
            "XDG_STATE_HOME",
            base.join("state").to_string_lossy().into_owned(),
        ),
        (
            "XDG_CACHE_HOME",
            base.join("cache").to_string_lossy().into_owned(),
        ),
    ])
}

/// Returns the path to the opencode.db under the AlfAlfa data directory.
pub fn opencode_db_path() -> Result<PathBuf, String> {
    let base = base_dir()?;
    Ok(base.join("data").join("opencode").join("opencode.db"))
}

/// Creates the ~/.alfalfa/ directory structure and seed files.
pub fn ensure_directories() -> Result<(), String> {
    let base = base_dir()?;
    let dirs = [
        base.clone(),
        base.join("data"),
        base.join("config").join("opencode"),
        base.join("state"),
        base.join("cache"),
    ];
    for dir in &dirs {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create directory {}: {e}", dir.display()))?;
    }

    // Copy global auth.json if it doesn't exist in alfalfa data
    inherit_global_auth(&base)?;

    // Write opencode config with Alfalfa identity
    write_identity_config(&base)?;

    // Write custom lf tool
    write_lf_tool(&base)?;

    Ok(())
}

/// Copies the user's global opencode auth.json into the AlfAlfa data directory.
fn inherit_global_auth(base: &Path) -> Result<(), String> {
    let dest = base.join("data").join("opencode").join("auth.json");
    if dest.exists() {
        return Ok(());
    }

    // Ensure the parent directory exists
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create auth directory: {e}"))?;
    }

    // Find global auth.json
    let xdg_data = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
        dirs::home_dir()
            .map(|h| {
                h.join(".local")
                    .join("share")
                    .to_string_lossy()
                    .into_owned()
            })
            .unwrap_or_default()
    });
    let src = PathBuf::from(&xdg_data).join("opencode").join("auth.json");
    if !src.exists() {
        return Ok(());
    }

    std::fs::copy(&src, &dest).map_err(|e| format!("Failed to copy auth.json: {e}"))?;
    tracing::info!("Copied global auth.json to AlfAlfa data directory");
    Ok(())
}

/// Writes the opencode.json config with Alfalfa system prompts.
/// Only writes if the file doesn't exist (preserves user edits).
fn write_identity_config(base: &Path) -> Result<(), String> {
    let config_path = base.join("config").join("opencode").join("opencode.json");
    if config_path.exists() {
        return Ok(());
    }

    let config = serde_json::json!({
        "agent": {
            "build": {
                "description": "Build mode - full tool access for creating and running evaluations",
                "mode": "primary",
                "prompt": SYSTEM_PROMPT_BUILD
            },
            "plan": {
                "description": "Plan mode - analyze and plan evaluations without modifying anything",
                "mode": "primary",
                "prompt": SYSTEM_PROMPT_PLAN
            }
        },
        "permission": {
            "read": "allow",
            "glob": "allow",
            "grep": "allow",
            "webfetch": "allow",
            "lf": "allow",
            "bash": "ask",
            "edit": "allow",
            "write": "allow"
        }
    });

    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    std::fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write opencode.json: {e}"))?;
    tracing::info!("Wrote AlfAlfa identity config to {}", config_path.display());
    Ok(())
}

/// Writes the custom `lf` tool and its package.json into the opencode config.
/// The tool is discovered by convention: opencode scans tools/ for .ts files.
/// Only writes if the tool file doesn't exist (preserves user edits).
fn write_lf_tool(base: &Path) -> Result<(), String> {
    let opencode_dir = base.join("config").join("opencode");
    let tools_dir = opencode_dir.join("tools");
    std::fs::create_dir_all(&tools_dir)
        .map_err(|e| format!("Failed to create tools directory: {e}"))?;

    // Write tools/lf.ts
    let tool_path = tools_dir.join("lf.ts");
    if !tool_path.exists() {
        std::fs::write(&tool_path, LF_TOOL_SOURCE)
            .map_err(|e| format!("Failed to write lf.ts: {e}"))?;
        tracing::info!("Wrote custom lf tool to {}", tool_path.display());
    }

    // Write package.json
    let pkg_path = opencode_dir.join("package.json");
    if !pkg_path.exists() {
        std::fs::write(&pkg_path, LF_TOOL_PACKAGE_JSON)
            .map_err(|e| format!("Failed to write package.json: {e}"))?;
        tracing::info!("Wrote lf tool package.json to {}", pkg_path.display());
    }

    // Install dependencies if node_modules doesn't exist.
    // Use /bin/sh explicitly -- the install script uses bash syntax (&&, ||)
    // which isn't compatible with all user shells (e.g. fish).
    let node_modules = opencode_dir.join("node_modules");
    if !node_modules.exists() {
        tracing::info!("Installing lf tool dependencies...");
        let install_cmd = format!(
            "cd {} && (bun install 2>/dev/null || npm install 2>/dev/null)",
            opencode_dir.to_string_lossy()
        );
        let output = std::process::Command::new("/bin/sh")
            .args(["-c", &install_cmd])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output();

        match output {
            Ok(o) if o.status.success() => {
                tracing::info!("lf tool dependencies installed");
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                tracing::warn!("Failed to install lf tool dependencies: {stderr}");
            }
            Err(e) => {
                tracing::warn!("Failed to run package install for lf tool: {e}");
            }
        }
    }

    // Write .gitignore
    let gitignore_path = opencode_dir.join(".gitignore");
    if !gitignore_path.exists() {
        std::fs::write(
            &gitignore_path,
            "node_modules\npackage.json\nbun.lock\npackage-lock.json\n.gitignore\n",
        )
        .map_err(|e| format!("Failed to write .gitignore: {e}"))?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Journal
// ---------------------------------------------------------------------------

/// Returns the journal directory for a given project path: <project>/.lf_agent/journal/
pub fn journal_dir_for_project(project_path: &str) -> PathBuf {
    PathBuf::from(project_path)
        .join(".lf_agent")
        .join("journal")
}

/// Ensures the journal directory and seed files exist for a project.
#[tauri::command]
#[specta::specta]
pub fn ensure_project_journal(project_path: String) -> Result<String, String> {
    let journal_dir = journal_dir_for_project(&project_path);
    std::fs::create_dir_all(&journal_dir)
        .map_err(|e| format!("Failed to create journal directory: {e}"))?;

    let seeds: &[(&str, &str)] = &[
        ("_schema.md", JOURNAL_SCHEMA),
        ("_log.md", JOURNAL_LOG_SEED),
    ];

    for (filename, content) in seeds {
        let file_path = journal_dir.join(filename);
        if !file_path.exists() {
            std::fs::write(&file_path, content)
                .map_err(|e| format!("Failed to write {filename}: {e}"))?;
        }
    }

    // overview.md gets a dynamic date
    let overview_path = journal_dir.join("overview.md");
    if !overview_path.exists() {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let overview = JOURNAL_OVERVIEW_TEMPLATE.replace("{{DATE}}", &today);
        std::fs::write(&overview_path, overview)
            .map_err(|e| format!("Failed to write overview.md: {e}"))?;
    }

    Ok(journal_dir.to_string_lossy().into_owned())
}

/// Returns the absolute journal directory path for a project.
#[tauri::command]
#[specta::specta]
pub fn get_journal_path(project_path: String) -> Result<String, String> {
    let journal_dir = journal_dir_for_project(&project_path);
    Ok(journal_dir.to_string_lossy().into_owned())
}

/// Reads the journal overview.md content for a project. Returns empty string if not found.
#[tauri::command]
#[specta::specta]
pub fn get_journal_overview(project_path: String) -> Result<String, String> {
    let journal_dir = journal_dir_for_project(&project_path);
    let overview_path = journal_dir.join("overview.md");
    if !overview_path.exists() {
        return Ok(String::new());
    }
    std::fs::read_to_string(&overview_path).map_err(|e| format!("Failed to read overview.md: {e}"))
}

// ---------------------------------------------------------------------------
// lf skills
// ---------------------------------------------------------------------------

/// Runs a command using the user's interactive login shell, matching
/// the same environment the sidecar sees (venv PATH etc.).
/// Tries `-il` first, falls back to `-l` if that fails.
fn run_in_user_shell(command: &str) -> Result<std::process::Output, String> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    // Try interactive login shell first (-il), same as sidecar probe
    let output = std::process::Command::new(&shell)
        .args(["-il", "-c", command])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run `{command}`: {e}"))?;

    if output.status.success() {
        return Ok(output);
    }

    // Fall back to login-only (-l)
    std::process::Command::new(&shell)
        .args(["-l", "-c", command])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run `{command}`: {e}"))
}

/// Runs `lf skills` using the user's login shell and returns the output.
/// Uses the same shell-probing mechanism as the sidecar spawn.
#[tauri::command]
#[specta::specta]
pub fn load_lf_skills() -> Result<String, String> {
    let output = run_in_user_shell("lf skills")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "lf skills failed (exit code {:?}): {}",
            output.status.code(),
            if stderr.is_empty() {
                "unknown error"
            } else {
                &stderr
            }
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Checks if the `lf` CLI is available on the user's PATH.
#[tauri::command]
#[specta::specta]
pub fn check_lf_available() -> Result<bool, String> {
    let output = run_in_user_shell("command -v lf")?;
    Ok(output.status.success())
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const SYSTEM_PROMPT_BUILD: &str = r#"IMPORTANT IDENTITY OVERRIDE: You are NOT OpenCode. You are Alfalfa, developed by LatticeFlow. Your name stands for "ALFALFA: LatticeFlow's Awesome Little Friendly Agent". You are powered by OpenCode as your underlying framework, but your identity is Alfalfa. When asked who you are, what you are, or what your name is, you MUST always identify yourself as "Alfalfa by LatticeFlow". Never refer to yourself as OpenCode.

You are an AI assistant specialized in LatticeFlow AI GO! evaluations.

Your purpose is to help users create, configure, test, and run AI model evaluations
using the `lf` CLI and YAML specifications.

Key behaviors:
- Use the `lf` tool (not raw bash) for all LatticeFlow operations.
- After creating or modifying YAML specs, always validate them with `lf test` before running.
- When creating entities (models, datasets, tasks), add them one at a time and verify each.
- Read existing project files before making changes to understand the current state.
- When the user describes an evaluation goal in natural language, translate it into the
  appropriate YAML specs and lf commands.
- Always explain what you're doing and why before executing commands.
- If credentials or configuration seem missing, ask the user rather than guessing.

You have full access to file operations, the lf CLI, and bash commands.

You have access to a persistent project journal in .lf_agent/journal/.
This journal tracks evaluation progress, discoveries, ideas, and decisions
across sessions. When relevant, you can read journal pages using the read
tool to understand past work. The user can trigger journal updates with
the /journal command — you don't need to update it proactively."#;

const SYSTEM_PROMPT_PLAN: &str = r#"IMPORTANT IDENTITY OVERRIDE: You are NOT OpenCode. You are Alfalfa in planning mode, developed by LatticeFlow. Your name stands for "ALFALFA: LatticeFlow's Awesome Little Friendly Agent". You are powered by OpenCode as your underlying framework, but your identity is Alfalfa. Never refer to yourself as OpenCode.

Analyze the user's evaluation needs and create a detailed plan. You can:
- Read files and explore the project structure
- Search for patterns in the codebase
- Analyze existing YAML specifications
- Suggest changes and new specifications

You CANNOT modify files, run commands, or execute lf operations in this mode.
Present your analysis and recommendations clearly so the user can switch to Build
mode to execute them.

You have access to a persistent project journal in .lf_agent/journal/.
This journal tracks evaluation progress, discoveries, ideas, and decisions
across sessions. When relevant, you can read journal pages using the read
tool to understand past work."#;

const JOURNAL_SCHEMA: &str = r#"# Journal Schema

This file defines the conventions Alfalfa follows when maintaining the project journal.

## Page Conventions

- **Naming**: kebab-case, descriptive (e.g. `toxicity-eval.md`, not `eval-1.md`)
- **Frontmatter**: Every page (except this file and `_log.md`) must have YAML frontmatter with:
  - `tags`: list of relevant tags
  - `created`: date (YYYY-MM-DD)
  - `updated`: date (YYYY-MM-DD), bump on every edit
  - `links`: list of related page names (without .md extension)
- **Links**: Use `[[page-name]]` Obsidian wikilink syntax in body text
- **Structure**: Use markdown headers to organize content within a page

## Special Files

- `_schema.md` (this file): Conventions. Maintain but never delete.
- `_log.md`: Chronological activity log. Append-only.
- `overview.md`: High-level project synthesis. Keep concise (< 2 pages). Update to reflect current state.

## Log Format

Each entry in `_log.md`:

```
## [YYYY-MM-DD HH:MM] verb | one-line summary

- Bullet points with details
```

## Update Guidelines

- When creating a page: add frontmatter, link to related pages, update those pages' links back
- When updating a page: always bump the `updated` date
- Keep `overview.md` as a concise synthesis — not a dump of everything, but a useful orientation for the next session
- Prefer updating existing pages over creating new ones for the same topic
- Use tags consistently across pages
"#;

const JOURNAL_LOG_SEED: &str = "# Activity Log\n\nChronological record of journal updates.\n";

const LF_TOOL_SOURCE: &str = r#"import { tool } from "@opencode-ai/plugin"
import { spawn } from "child_process"

export default tool({
  description: `Execute LatticeFlow AI GO! CLI commands.

Use this tool for all lf operations including:
- Adding entities: lf add model -f model.yaml
- Listing entities: lf list model
- Running evaluations: lf run -f run.yaml
- Testing: lf test model <key>
- Managing AI apps: lf app list, lf switch <key>
- Checking status: lf status
- Exporting: lf export eval --id <id> -o ./results

The 'lf' prefix is included automatically. Just provide the subcommand and arguments.`,
  args: {
    command: tool.schema
      .string()
      .describe(
        "The lf subcommand and arguments. Examples: 'add model -f model.yaml', 'run -f run.yaml', 'list model', 'test model my-model', 'status'"
      ),
  },
  async execute(args) {
    return new Promise((resolve) => {
      const parts = args.command.split(/\s+/).filter(Boolean)
      const proc = spawn("lf", parts, { stdio: ["ignore", "pipe", "pipe"] })

      let stdout = ""
      let stderr = ""

      proc.stdout.on("data", (chunk) => { stdout += chunk.toString() })
      proc.stderr.on("data", (chunk) => { stderr += chunk.toString() })

      proc.on("close", (exitCode) => {
        if (exitCode !== 0) {
          resolve(`Command 'lf ${args.command}' failed (exit code ${exitCode}):\n${stderr || stdout}`)
        } else {
          resolve(stdout || "(command completed with no output)")
        }
      })

      proc.on("error", (err) => {
        resolve(`Failed to spawn 'lf': ${err.message}`)
      })
    })
  },
})
"#;

const LF_TOOL_PACKAGE_JSON: &str = r#"{
  "dependencies": {
    "@opencode-ai/plugin": "1.4.6"
  }
}
"#;

const JOURNAL_OVERVIEW_TEMPLATE: &str = r#"---
tags: [overview]
created: {{DATE}}
updated: {{DATE}}
links: []
---

# Project Overview

This journal has just been initialized. As the project progresses, this page will be
updated with a high-level synthesis of the evaluation project — what's been tried,
what works, current status, and next steps.
"#;
