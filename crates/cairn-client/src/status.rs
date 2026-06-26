//! `cairn status` - show server connection, token info, and agent status.

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct Status {
    version: String,
    server: Option<String>,
    token: Option<TokenInfo>,
    agents: Vec<String>,
}

#[derive(Debug, Serialize)]
struct TokenInfo {
    name: String,
    scope: String,
    valid: bool,
    expires: Option<String>,
}

pub fn run(json_output: bool) -> Result<()> {
    let server = std::env::var("CAIRN_SERVER")
        .ok()
        .filter(|s| !s.trim().is_empty());
    let token = std::env::var("CAIRN_TOKEN").ok().filter(|t| !t.is_empty());

    // Decode JWT to extract claim info (no signature verification needed).
    let mut token_info = token.as_deref().and_then(decode_jwt_info);

    // Verify the token against the server.
    if let (Some(ref srv), Some(ref tok)) = (&server, token.as_deref()) {
        if let Some(ref mut info) = token_info {
            let url = format!("{}/api/memory/wakeup?limit=1", srv.trim_end_matches('/'));
            match ureq::get(&url)
                .set("Authorization", &format!("Bearer {tok}"))
                .call()
            {
                Ok(resp) => {
                    info.valid = resp.status() == 200;
                }
                Err(_) => {
                    info.valid = false;
                }
            }
        }
    }

    // Detect configured agents.
    let agents = detect_agents();

    let status = Status {
        version: env!("CARGO_PKG_VERSION").to_string(),
        server,
        token: token_info,
        agents,
    };

    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&status).unwrap_or_default()
        );
    } else {
        println!("Cairn client v{}", env!("CARGO_PKG_VERSION"));
        match &status.server {
            Some(s) => println!("Server:     {s}"),
            None => println!("Server:     (not configured)"),
        }
        match &status.token {
            Some(t) => {
                let valid = if t.valid { "valid" } else { "INVALID" };
                println!("Token:      {} ({} scope, {valid})", t.name, t.scope);
                if let Some(exp) = &t.expires {
                    println!("Expires:    {exp}");
                }
            }
            None => println!("Token:      (not configured)"),
        }
        if status.agents.is_empty() {
            println!("Agents:     (none detected)");
        } else {
            println!("Agents:     {}", status.agents.join(", "));
        }
        if status.server.is_none() || status.token.is_none() {
            println!("\nRun `cairn onboard --server <url> --token <jwt>` to configure.");
        }
    }

    Ok(())
}

/// Decode the JWT payload (middle section) without signature verification
/// to extract token name and scope for display.
fn decode_jwt_info(jwt: &str) -> Option<TokenInfo> {
    let payload_b64 = jwt.split('.').nth(1)?;
    // Add padding if needed
    let padded = match payload_b64.len() % 4 {
        2 => format!("{payload_b64}=="),
        3 => format!("{payload_b64}="),
        _ => payload_b64.to_string(),
    };
    let bytes = base64_decode(&padded)?;
    #[derive(Deserialize)]
    struct Claims {
        sub: String,
        scope: String,
        #[serde(default)]
        exp: Option<i64>,
    }
    let claims: Claims = serde_json::from_slice(&bytes).ok()?;
    let expires = claims.exp.map(|e| {
        let dt = chrono::DateTime::from_timestamp(e, 0).unwrap_or_default();
        dt.format("%Y-%m-%d %H:%M UTC").to_string()
    });
    Some(TokenInfo {
        name: claims.sub,
        scope: claims.scope,
        valid: false, // will be set by server check
        expires,
    })
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(s)
        .ok()
}

fn detect_agents() -> Vec<String> {
    let project = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let home = home_dir();
    let mut found = Vec::new();
    for (id, check) in [
        (
            "claude-code",
            project.join(".claude").exists()
                || project.join(".mcp.json").exists()
                || home
                    .as_deref()
                    .is_some_and(|h| h.join(".claude").exists() || h.join(".claude.json").exists()),
        ),
        (
            "codex",
            codex_config_path(home.as_deref()).exists()
                || project.join(".codex").join("config.toml").exists()
                || home
                    .as_deref()
                    .is_some_and(|h| h.join(".codex/config.toml").exists()),
        ),
        (
            "opencode",
            opencode_config_path().exists() || project.join(".opencode").exists(),
        ),
    ] {
        if check {
            found.push(id.to_string());
        }
    }
    found
}

use std::path::{Path, PathBuf};

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

fn codex_config_path(home: Option<&Path>) -> PathBuf {
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| home.map(PathBuf::from))
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));
    config_home.join(".codex").join("config.toml")
}
