# Cairn — Claude Code plugin

One install wires Cairn into Claude Code: the **MCP server** (16 tools), the four **lifecycle
hooks** (SessionStart/UserPromptSubmit/PostToolUse/SessionEnd), **slash commands**
(`/cairn:recall`, `/cairn:remember`, `/cairn:sanitize`, `/cairn:bench`), and a **usage skill** that
tells the model when to reach for Cairn.

## Prerequisite

Install the `cairn` binary so it's on your `PATH` (the plugin shells out to it):

```sh
curl -fsSL https://raw.githubusercontent.com/Vellixia/Cairn/main/scripts/install.sh | sh   # Linux/macOS
# Windows: irm https://raw.githubusercontent.com/Vellixia/Cairn/main/scripts/install.ps1 | iex
# or: docker compose up   ·   or: cargo install --git https://github.com/Vellixia/Cairn cairn-cli
```

## Install the plugin

```text
/plugin marketplace add Vellixia/Cairn
/plugin install cairn@cairn
```

That's it — open a session and Cairn's memory, lean reads, guardrails, and commands are live.
Manage with `/plugin list`, `/plugin disable cairn@cairn`, `/plugin update cairn@cairn`.
