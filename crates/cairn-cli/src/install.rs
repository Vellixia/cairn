//! `cairn install <agent>` — wire an agent up to this Cairn install.
//!
//! For Claude Code this merges (non-destructively) two project files:
//!
//! - `.mcp.json` registers the `cairn` MCP server.
//! - `.claude/settings.json` adds SessionStart + UserPromptSubmit hooks calling `cairn hook`.
//!
//! Existing content is preserved; our entries are added idempotently.

use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;

pub fn run(agent: Option<&str>, all: bool) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let agents: Vec<&str> = if all {
        vec!["claude-code"]
    } else {
        vec![agent.unwrap_or("claude-code")]
    };
    for a in agents {
        match a {
            "claude-code" | "claude" => install_claude_code(&cwd)?,
            other => println!("cairn: agent '{other}' is not supported yet — coming soon."),
        }
    }
    Ok(())
}

fn install_claude_code(dir: &Path) -> Result<()> {
    // 1) Register the MCP server in .mcp.json.
    let mcp_path = dir.join(".mcp.json");
    let mut mcp = read_object(&mcp_path)?;
    let servers = mcp
        .entry("mcpServers")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .context(".mcp.json: mcpServers is not an object")?;
    servers.insert(
        "cairn".into(),
        json!({ "command": "cairn", "args": ["mcp"] }),
    );
    write_json(&mcp_path, &Value::Object(mcp))?;

    // 2) Add lifecycle hooks in .claude/settings.json.
    let settings_path = dir.join(".claude").join("settings.json");
    let mut settings = read_object(&settings_path)?;
    {
        let hooks = settings
            .entry("hooks")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .context("settings.json: hooks is not an object")?;
        add_hook(hooks, "SessionStart", "cairn hook SessionStart", None);
        add_hook(
            hooks,
            "UserPromptSubmit",
            "cairn hook UserPromptSubmit",
            None,
        );
        add_hook(
            hooks,
            "PostToolUse",
            "cairn hook PostToolUse",
            Some("Edit|Write|MultiEdit|NotebookEdit"),
        );
        add_hook(hooks, "SessionEnd", "cairn hook SessionEnd", None);
    }
    write_json(&settings_path, &Value::Object(settings))?;

    println!("\u{2713} Configured Claude Code:");
    println!("  - {}  (MCP server: cairn)", mcp_path.display());
    println!(
        "  - {}  (hooks: SessionStart, UserPromptSubmit, PostToolUse, SessionEnd)",
        settings_path.display()
    );
    println!("\nStart the server with `cairn serve`, then open a Claude Code session here.");
    Ok(())
}

/// Add a command hook for `event` (optionally scoped to a tool `matcher`) if an identical one
/// isn't already present.
fn add_hook(hooks: &mut Map<String, Value>, event: &str, command: &str, matcher: Option<&str>) {
    let groups = hooks
        .entry(event)
        .or_insert_with(|| json!([]))
        .as_array_mut();
    let Some(groups) = groups else { return };

    let already = groups.iter().any(|g| {
        g.get("hooks").and_then(Value::as_array).is_some_and(|hs| {
            hs.iter()
                .any(|h| h.get("command").and_then(Value::as_str) == Some(command))
        })
    });
    if !already {
        let mut group = json!({ "hooks": [{ "type": "command", "command": command }] });
        if let Some(m) = matcher {
            group["matcher"] = json!(m);
        }
        groups.push(group);
    }
}

/// Read a JSON object from `path`, or an empty object if missing/blank/non-object.
fn read_object(path: &Path) -> Result<Map<String, Value>> {
    if !path.exists() {
        return Ok(Map::new());
    }
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    if text.trim().is_empty() {
        return Ok(Map::new());
    }
    let value: Value =
        serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
    Ok(value.as_object().cloned().unwrap_or_default())
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{text}\n")).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_is_idempotent_and_non_destructive() {
        let dir = tempfile::tempdir().unwrap();
        // Pre-existing settings with an unrelated hook and key.
        fs::create_dir_all(dir.path().join(".claude")).unwrap();
        fs::write(
            dir.path().join(".claude/settings.json"),
            r#"{"model":"opus","hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"echo hi"}]}]}}"#,
        )
        .unwrap();

        install_claude_code(dir.path()).unwrap();
        // Run twice — must not duplicate our hook.
        install_claude_code(dir.path()).unwrap();

        let settings: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join(".claude/settings.json")).unwrap(),
        )
        .unwrap();
        // Preserved the unrelated key.
        assert_eq!(settings["model"], "opus");
        let starts = settings["hooks"]["SessionStart"].as_array().unwrap();
        // Original echo hook + our cairn hook, exactly once each.
        let cairn_count = starts
            .iter()
            .filter(|g| g["hooks"][0]["command"] == "cairn hook SessionStart")
            .count();
        assert_eq!(cairn_count, 1, "cairn hook must be added exactly once");
        assert!(starts.iter().any(|g| g["hooks"][0]["command"] == "echo hi"));
        assert!(
            settings["hooks"]["PostToolUse"]
                .as_array()
                .unwrap()
                .iter()
                .any(|g| g["hooks"][0]["command"] == "cairn hook PostToolUse"),
            "PostToolUse hook must be installed"
        );

        let mcp: Value =
            serde_json::from_str(&fs::read_to_string(dir.path().join(".mcp.json")).unwrap())
                .unwrap();
        assert_eq!(mcp["mcpServers"]["cairn"]["command"], "cairn");
    }
}
