//! The minimal branded page served at `/` when `web/out/` is missing (no Next.js build).
//!
//! Intentionally diagnostic-only — never calls any authed endpoint. The only network call it
//! makes is to the public `/api/health` and `/api/auth/status` so the page can tell the user
//! whether to go to `/login` (admin already configured) or `/setup` (first run).
//!
//! Once the dashboard is built and embedded via `rust-embed`, this fallback is no longer served
//! (the Next.js export serves `/` instead). It exists for fresh checkouts and the
//! `cargo run -p cairn-server -- serve` zero-deps path.

pub const INDEX_HTML: &str = r###"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>Cairn — context & reliability for AI agents</title>
<style>
  :root{
    --ink:#0B0F14; --surface:#12181F; --surface2:#1a2129; --slate:#8A94A6;
    --offwhite:#ECEFF4; --ember:#FB923C; --teal:#2DD4BF; --line:#222b35;
  }
  *{box-sizing:border-box}
  body{margin:0;background:radial-gradient(1200px 600px at 70% -10%, #16202b 0%, var(--ink) 55%);
    color:var(--offwhite);font:15px/1.55 ui-sans-serif,system-ui,-apple-system,Segoe UI,Roboto,Inter,sans-serif;
    min-height:100vh}
  .wrap{max-width:760px;margin:0 auto;padding:48px 20px 80px}
  header{display:flex;align-items:center;gap:14px;margin-bottom:24px}
  .logo{width:44px;height:44px;flex:0 0 auto}
  h1{font-size:30px;margin:0;letter-spacing:-.02em}
  .tag{color:var(--slate);margin:4px 0 0;font-size:14px}
  .card{background:var(--surface);border:1px solid var(--line);border-radius:14px;padding:20px;margin-bottom:16px}
  .pill{display:inline-block;border:1px solid var(--line);background:var(--surface2);
    border-radius:999px;padding:4px 10px;font-size:12.5px;color:#b9c2cf;margin-right:6px}
  .cta{display:inline-block;margin-top:14px;background:var(--ember);color:#1a1206;border:0;
    border-radius:9px;padding:10px 18px;font-weight:700;text-decoration:none;font-size:14px}
  .cta.ghost{background:transparent;color:var(--offwhite);border:1px solid var(--line);margin-left:8px}
  pre.codeblock{background:var(--surface2);border:1px solid var(--line);border-radius:9px;
    padding:12px 14px;font:12.5px/1.5 ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;
    color:#cdd5e0;overflow:auto;margin:10px 0 0}
  .stat{display:flex;justify-content:space-between;padding:4px 0;font-size:13.5px;
    border-bottom:1px dashed var(--line)}
  .stat:last-child{border-bottom:0}
  .stat b{color:var(--teal);font-variant-numeric:tabular-nums;font-weight:600}
  footer{margin-top:32px;color:var(--slate);font-size:12.5px}
  a{color:var(--teal)}
</style>
</head>
<body>
<div class="wrap">
  <header>
    <svg class="logo" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
      <ellipse cx="24" cy="38" rx="15" ry="2.6" fill="#000000" opacity="0.28"/>
      <ellipse cx="24" cy="34" rx="15" ry="5" fill="#6B7689"/>
      <ellipse cx="22.5" cy="28" rx="12" ry="4.2" fill="#8A94A6"/>
      <ellipse cx="25.5" cy="23" rx="9" ry="3.5" fill="#AEB8C6"/>
      <ellipse cx="24" cy="18" rx="6" ry="3.3" fill="#FB923C"/>
    </svg>
    <div>
      <h1>Cairn</h1>
      <p class="tag">Self-hosted context &amp; reliability layer for AI agents.</p>
    </div>
  </header>

  <div class="card">
    <h2 style="margin:0 0 6px;font-size:16px">Server status</h2>
    <div class="stat"><span>Health</span><b id="health">checking…</b></div>
    <div class="stat"><span>Version</span><b id="version">…</b></div>
    <div class="stat"><span>Admin</span><b id="admin">checking…</b></div>
    <div class="stat"><span>Next step</span><b id="next">…</b></div>
  </div>

  <div class="card">
    <h2 style="margin:0 0 6px;font-size:16px">Open the dashboard</h2>
    <p style="color:var(--slate);margin:0 0 6px;font-size:13.5px">
      The full dashboard ships with the Next.js build. If you see this page, the
      binary is serving its built-in fallback because <code>web/out/</code> is missing.
    </p>
    <a id="primaryCta" class="cta" href="/login">Open dashboard</a>
    <a class="cta ghost" href="/api/health" target="_blank" rel="noopener">/api/health</a>
    <pre class="codeblock">curl -fsSL https://raw.githubusercontent.com/Vellixia/cairn/main/scripts/install.sh | sh</pre>
  </div>

  <div class="card">
    <h2 style="margin:0 0 6px;font-size:16px">Issue a device token (CLI)</h2>
    <p style="color:var(--slate);margin:0 0 6px;font-size:13.5px">
      From the server host:
    </p>
    <pre class="codeblock">cairn-server token create my-laptop --scope write</pre>
    <p style="color:var(--slate);margin:10px 0 0;font-size:13px">
      Then on the device:&nbsp;
      <code>cairn sync --server http://&lt;host&gt;:7777 --token &lt;jwt&gt;</code>
    </p>
  </div>

  <footer>🪨 Cairn · Apache-2.0 · every traveler adds a stone.</footer>
</div>

<script>
const $ = (id) => document.getElementById(id);
async function probe(){
  // /api/health is the only endpoint we ever call from the fallback page. It is public.
  let healthy = false;
  try {
    const h = await (await fetch('/api/health')).json();
    $('health').textContent = h.status;
    $('version').textContent = 'v' + h.version;
    healthy = h.status === 'ok';
  } catch (e) {
    $('health').textContent = 'unreachable';
  }
  if (!healthy) {
    $('admin').textContent = 'unknown';
    $('next').textContent = 'start the server';
    $('primaryCta').textContent = 'Start server';
    $('primaryCta').removeAttribute('href');
    return;
  }
  try {
    const a = await (await fetch('/api/auth/status')).json();
    $('admin').textContent = a.admin_exists ? 'configured' : 'not configured';
    if (a.setup_required) {
      $('next').textContent = 'first-run setup';
      $('primaryCta').textContent = 'Create admin';
      $('primaryCta').href = '/setup';
    } else {
      $('next').textContent = 'sign in';
      $('primaryCta').textContent = 'Sign in';
      $('primaryCta').href = '/login';
    }
  } catch (e) {
    $('admin').textContent = 'unknown';
    $('next').textContent = 'sign in';
  }
}
probe();
</script>
</body>
</html>"###;
