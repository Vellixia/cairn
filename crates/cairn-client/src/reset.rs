//! `cairn reset` - remove Cairn-managed entries from all agent config files.

use anyhow::Result;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run(dry_run: bool) -> Result<()> {
    let project = std::env::current_dir()?;
    let home = home_dir();

    let mut removed = 0usize;

    // Remove managed block from CLAUDE.md
    if project.join("CLAUDE.md").exists() {
        if let Ok(t) = fs::read_to_string(project.join("CLAUDE.md")) {
            if let Some(cleaned) = remove_managed_block(&t) {
                if cleaned != t {
                    removed += 1;
                    if dry_run {
                        println!("Would remove managed block from: CLAUDE.md");
                    } else {
                        fs::write(project.join("CLAUDE.md"), cleaned)?;
                        println!("Removed managed block from: CLAUDE.md");
                    }
                }
            }
        }
    }

    // Remove managed block from AGENTS.md
    if project.join("AGENTS.md").exists() {
        if let Ok(t) = fs::read_to_string(project.join("AGENTS.md")) {
            if let Some(cleaned) = remove_managed_block(&t) {
                if cleaned != t {
                    removed += 1;
                    if dry_run {
                        println!("Would remove managed block from: AGENTS.md");
                    } else {
                        fs::write(project.join("AGENTS.md"), cleaned)?;
                        println!("Removed managed block from: AGENTS.md");
                    }
                }
            }
        }
    }

    // Remove cairn entry from project .mcp.json
    if let Ok(mut cfg) = read_object(&project.join(".mcp.json")) {
        if let Some(servers) = cfg.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
            if servers.remove("cairn").is_some() {
                removed += 1;
                if dry_run {
                    println!("Would remove cafrom: .mcp.json");
                } else {
                    write_json(&project.join(".mcp.json"), &json!(cfg))?;
                    println!("Removed cafrom: .mcp.json");
                }
            }
        }
    }

    // Remove hooks from Claude Code settings.json
    if let Ok(mut settings) = read_object(&project.join(".claude/settings.json")) {
        if let Some(hooks) = settings.get_mut("hooks").and_then(|v| v.as_object_mut()) {
            let managed_events = [
                "SessionStart",
                "UserPromptSubmit",
                "PostToolUse",
                "SessionEnd",
            ];
            for event in &managed_events {
                hooks.remove(*event);
            }
            if hooks.is_empty() {
                settings.remove("hooks");
            }
            removed += 1;
            if dry_run {
                println!("Would remove Cairn hooks from: .claude/settings.json");
            } else {
                write_json(&project.join(".claude/settings.json"), &json!(settings))?;
                println!("Removed Cairn hooks from: .claude/settings.json");
            }
        }
    }

    // Remove Cairn hooks from Codex hooks.json (preserve foreign hooks)
    if let Some(h) = home.as_deref() {
        let codex_hooks = h.join(".codex").join("hooks.json");
        if let Ok(mut hooks_cfg) = read_object(&codex_hooks) {
            if let Some(hooks_obj) = hooks_cfg.get_mut("hooks").and_then(|v| v.as_object_mut()) {
                let mut cleaned = false;
                for events in hooks_obj.values_mut() {
                    if let Some(arr) = events.as_array_mut() {
                        let before = arr.len();
                        arr.retain(|entry| !is_cairn_hook(entry));
                        if arr.len() < before {
                            cleaned = true;
                        }
                    }
                }
                // Remove empty arrays
                hooks_obj.retain(|_, v| v.as_array().is_some_and(|a| !a.is_empty()));
                if cleaned {
                    removed += 1;
                    if dry_run {
                        println!("Would remove Cairn hooks from: {}", codex_hooks.display());
                    } else {
                        write_json(&codex_hooks, &json!(hooks_cfg))?;
                        println!("Removed Cairn hooks from: {}", codex_hooks.display());
                    }
                }
            }
        }

        // Remove cairn MCP entry from Codex config.toml
        let codex_config = h.join(".codex").join("config.toml");
        if codex_config.exists() {
            if let Ok(toml_str) = fs::read_to_string(&codex_config) {
                if toml_str.contains("[mcp_servers.cairn]") {
                    let cleaned = remove_codex_cairn_block(&toml_str);
                    if cleaned != toml_str {
                        removed += 1;
                        if dry_run {
                            println!("Would remove cairn MCP from: {}", codex_config.display());
                        } else {
                            fs::write(&codex_config, cleaned)?;
                            println!("Removed cairn MCP from: {}", codex_config.display());
                        }
                    }
                }
            }
        }

        // Remove OpenCode cairn MCP entry
        let oc_path = opencode_config_path();
        if let Ok(mut oc_cfg) = read_object(&oc_path) {
            if let Some(mcp) = oc_cfg.get_mut("mcp").and_then(|v| v.as_object_mut()) {
                if mcp.remove("cairn").is_some() {
                    removed += 1;
                    if dry_run {
                        println!("Would remove cafrom: {}", oc_path.display());
                    } else {
                        write_json(&oc_path, &json!(oc_cfg))?;
                        println!("Removed cafrom: {}", oc_path.display());
                    }
                }
            }
        }

        // Remove OpenCode Cairn plugin
        let plugin_path = h.join(".config/opencode/plugins/cairn.js");
        if plugin_path.exists() {
            removed += 1;
            if dry_run {
                println!("Would remove: {}", plugin_path.display());
            } else {
                fs::remove_file(&plugin_path)?;
                println!("Removed: {}", plugin_path.display());
            }
        }
    }

    if removed == 0 {
        println!("No Cairn-managed entries found.");
    } else if dry_run {
        println!("\nRun `cairn reset` without --dry-run to apply.");
    } else {
        println!("\nRemoved {} Cairn entries.", removed);
    }
    Ok(())
}

fn is_cairn_hook(entry: &serde_json::Value) -> bool {
    entry
        .get("hooks")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|h| h.get("command"))
        .and_then(|c| c.as_str())
        .is_some_and(|c| c.starts_with("cairn hook"))
}

fn remove_managed_block(text: &str) -> Option<String> {
    let begin = "<!-- BEGIN CAIRN";
    let end = "<!-- END CAIRN -->";
    let start = text.find(begin)?;
    let end_pos = text[start..].find(end)?;
    let before = &text[..start];
    let after = &text[start + end_pos + end.len()..];
    let cleaned = format!("{}{}", before.trim_end(), after);
    if cleaned.trim().is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn remove_codex_cairn_block(toml: &str) -> String {
    let mut out = String::new();
    let mut skip = false;
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed == "[mcp_servers.cairn]" {
            skip = true;
            continue;
        }
        if skip && trimmed.starts_with('[') && trimmed != "[mcp_servers.cairn.env]" {
            skip = false;
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if skip {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out.trim_end().to_string()
}

fn read_object(path: &Path) -> Result<serde_json::Map<String, serde_json::Value>> {
    if !path.exists() {
        return Ok(serde_json::Map::new());
    }
    let text = fs::read_to_string(path)?;
    if text.trim().is_empty() {
        return Ok(serde_json::Map::new());
    }
    let value: serde_json::Value = serde_json::from_str(&text)?;
    Ok(value.as_object().cloned().unwrap_or_default())
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{text}\n"))?;
    Ok(())
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn opencode_config_path() -> PathBuf {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("opencode").join("opencode.json");
    }
    let base = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .or_else(home_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(".config").join("opencode").join("opencode.json")
}
