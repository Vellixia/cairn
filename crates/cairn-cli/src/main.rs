//! The `cairn` binary.
//!
//! `cairn serve` starts the server + embedded web UI. The other subcommands operate directly on
//! the local store so you can poke at the engine without a running server (handy for tests).

use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Context;
use cairn_api::AppState;
use cairn_core::{Config, NewMemory};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "cairn",
    version,
    about = "The context & reliability layer for AI agents"
)]
struct Cli {
    /// Override the data directory (defaults to the OS data dir; use /data in Docker).
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Start the Cairn server (HTTP API + web control plane).
    Serve {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        #[arg(long, default_value_t = 7777)]
        port: u16,
    },
    /// Store a memory.
    Remember { content: String },
    /// Recall memories matching a query.
    Recall {
        query: String,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Print the session-start bootstrap (wakeup) memories.
    Wakeup {
        #[arg(long, default_value_t = 12)]
        limit: usize,
    },
    /// Show basic stats.
    Stats,
    /// Verify the local setup.
    Doctor,
    /// Pair this device with a Cairn server using a code from the web UI. (coming soon)
    Pair { code: String },
    /// Configure an agent (or --all detected agents) to use this server. (coming soon)
    Install {
        /// Agent name, e.g. claude-code, codex, cursor. Omit with --all.
        agent: Option<String>,
        /// Configure every detected agent.
        #[arg(long)]
        all: bool,
    },
    /// Log in to a Cairn server. (coming soon)
    Login {
        /// Server URL.
        server: Option<String>,
    },
    /// Update the cairn binary in place. (coming soon)
    Update,
    /// Run the MCP server over stdio (point your agent's MCP config at `cairn mcp`).
    Mcp,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Log to stderr so `cairn mcp` keeps stdout clean for the JSON-RPC protocol.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_target(false)
        .compact()
        .init();

    let cli = Cli::parse();
    let cfg = Config::resolve(cli.data_dir).context("resolving data dir")?;

    match cli.cmd {
        Cmd::Serve { host, port } => {
            let state = AppState::new(&cfg)?;
            let addr: SocketAddr = format!("{host}:{port}")
                .parse()
                .with_context(|| format!("invalid address {host}:{port}"))?;
            println!("🪨  Cairn serving on http://{addr}");
            println!("    data dir: {}", cfg.data_dir().display());
            cairn_api::serve(addr, state).await?;
        }
        Cmd::Remember { content } => {
            let state = AppState::new(&cfg)?;
            let m = state.mem.remember(NewMemory::new(content))?;
            println!(
                "remembered {} ({}/{})",
                &m.id[..8],
                m.kind.as_str(),
                m.tier.as_str()
            );
        }
        Cmd::Recall { query, limit } => {
            let state = AppState::new(&cfg)?;
            let hits = state.mem.recall(&query, limit)?;
            if hits.is_empty() {
                println!("(no matches)");
            }
            for h in hits {
                println!("[{:.2}] {}", h.score, h.memory.content);
            }
        }
        Cmd::Wakeup { limit } => {
            let state = AppState::new(&cfg)?;
            for m in state.mem.wakeup(limit)? {
                println!("· ({}) {}", m.kind.as_str(), m.content);
            }
        }
        Cmd::Stats => {
            let state = AppState::new(&cfg)?;
            println!("memories: {}", state.store.count_memories()?);
        }
        Cmd::Doctor => {
            let _ = AppState::new(&cfg)?;
            println!("cairn doctor: ok");
            println!("  data dir : {}", cfg.data_dir().display());
            println!("  database : {}", cfg.db_path().display());
            println!("  blobs    : {}", cfg.blobs_dir().display());
        }
        Cmd::Pair { code } => coming_soon(&format!("pairing this device with code {code}")),
        Cmd::Install { agent, all } => {
            if all {
                coming_soon("auto-detecting and configuring all installed agents");
            } else if let Some(agent) = agent {
                coming_soon(&format!("configuring agent `{agent}`"));
            } else {
                coming_soon("pass an agent name or --all");
            }
        }
        Cmd::Login { server } => coming_soon(&format!(
            "logging in to {}",
            server.as_deref().unwrap_or("<server>")
        )),
        Cmd::Update => coming_soon("self-updating the cairn binary"),
        Cmd::Mcp => {
            // No stdout banner here: stdout is the MCP channel.
            let server = cairn_mcp::McpServer::new(&cfg)?;
            server.serve_stdio()?;
        }
    }
    Ok(())
}

/// Friendly placeholder for commands whose full behavior arrives in a later phase.
fn coming_soon(what: &str) {
    println!("cairn: {what} — coming soon in a later build.");
}
