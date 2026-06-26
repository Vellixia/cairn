//! AI agent lifecycle hook handler (`cairn hook <event>`).
//!
//! Supports Claude Code, Codex CLI, and OpenCode (via plugin bridge).
//! Reads JSON payload from stdin, calls the Cairn server HTTP API, and
//! emits additionalContext JSON on stdout per the agent's hook contract.
//!
//! Hooks must never break the agent: errors go to stderr, exit code is
//! always 0.

use anyhow::Result;
use serde_json::{json, Value};
use std::io::Read;

pub fn run(event: &str) -> Result<()> {
    if let Err(e) = run_inner(event) {
        eprintln!("cairn hook: {e}");
    }
    Ok(())
}

fn run_inner(event: &str) -> Result<()> {
    let server = std::env::var("CAIRN_SERVER")
        .ok()
        .filter(|s| !s.trim().is_empty());
    let token = std::env::var("CAIRN_TOKEN").ok().filter(|t| !t.is_empty());

    let (Some(server), Some(token)) = (server, token) else {
        eprintln!("cairn hook: CAIRN_SERVER or CAIRN_TOKEN not set. Hook skipped.");
        return Ok(());
    };

    let mut input = String::new();
    let _ = std::io::stdin().read_to_string(&mut input);
    let payload: Value = serde_json::from_str(input.trim()).unwrap_or(Value::Null);

    let rc = RemoteClient::new(&server, &token);
    rc.dispatch(event, &payload)
}

struct RemoteClient {
    server: String,
    token: String,
}

impl RemoteClient {
    fn new(server: &str, token: &str) -> Self {
        Self {
            server: server.trim_end_matches('/').to_string(),
            token: token.to_string(),
        }
    }

    fn get(&self, path: &str) -> ureq::Request {
        ureq::get(&format!("{}{}", self.server, path))
            .set("Authorization", &format!("Bearer {}", self.token))
    }

    fn post(&self, path: &str) -> ureq::Request {
        ureq::post(&format!("{}{}", self.server, path))
            .set("Authorization", &format!("Bearer {}", self.token))
    }

    fn dispatch(&self, event: &str, payload: &Value) -> Result<()> {
        match event {
            "SessionStart" => {
                let mut ctx = String::new();
                if let Ok(resp) = self.get("/api/guard/anchor").call() {
                    if let Ok(v) = resp.into_json::<Value>() {
                        if let Some(anchor) = v.get("anchor").and_then(Value::as_str) {
                            ctx.push_str(&format!("Current task: {anchor}\n\n"));
                        }
                    }
                }
                if let Ok(resp) = self.get("/api/profile").call() {
                    if let Ok(mems) = resp.into_json::<Vec<Value>>() {
                        if !mems.is_empty() {
                            ctx.push_str("Standing preferences:\n");
                            for m in &mems {
                                if let Some(c) = m.get("content").and_then(Value::as_str) {
                                    ctx.push_str(&format!("- {c}\n"));
                                }
                            }
                            ctx.push('\n');
                        }
                    }
                }
                if let Ok(resp) = self.get("/api/memory/wakeup").query("limit", "12").call() {
                    if let Ok(mems) = resp.into_json::<Vec<Value>>() {
                        let non_pref: Vec<_> = mems
                            .iter()
                            .filter(|m| m.get("kind").and_then(Value::as_str) != Some("preference"))
                            .collect();
                        if !non_pref.is_empty() {
                            ctx.push_str("Cairn memory:\n");
                            for m in non_pref {
                                let kind = m.get("kind").and_then(Value::as_str).unwrap_or("note");
                                let content =
                                    m.get("content").and_then(Value::as_str).unwrap_or("");
                                ctx.push_str(&format!("- ({kind}) {content}\n"));
                            }
                        }
                    }
                }
                if !ctx.is_empty() {
                    emit(event, &ctx);
                }
            }
            "UserPromptSubmit" => {
                let prompt = payload.get("prompt").and_then(Value::as_str).unwrap_or("");
                if prompt.trim().is_empty() {
                    return Ok(());
                }
                if let Ok(resp) = self
                    .get("/api/context/assemble")
                    .query("q", prompt)
                    .query("budget", "1200")
                    .call()
                {
                    if let Ok(v) = resp.into_json::<Value>() {
                        if v.get("included")
                            .and_then(Value::as_array)
                            .is_some_and(|a| !a.is_empty())
                        {
                            if let Some(ctx) = v.get("context").and_then(Value::as_str) {
                                if !ctx.is_empty() {
                                    emit(event, ctx);
                                }
                            }
                        }
                    }
                }
                let _ = self.post("/api/memory").send_json(json!({
                    "content": prompt,
                    "kind": "note",
                    "tier": "episodic",
                    "importance": 0.3
                }));
            }
            "SessionEnd" => {
                let _ = self.post("/api/memory/consolidate").send_json(json!({}));
            }
            _ => {
                // PostToolUse and other events are not proxied in remote-only mode.
            }
        }
        Ok(())
    }
}

/// Emit a context-injection payload on stdout per the agent hook contract.
fn emit(event: &str, context: &str) {
    let out = json!({
        "hookSpecificOutput": {
            "hookEventName": event,
            "additionalContext": context,
        }
    });
    println!("{out}");
}
