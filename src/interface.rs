use serde::Serialize;
use std::sync::{Arc, Mutex};
use tiny_http::{Header, Method, Response, Server};

use crate::{
    cache::CacheState,
    memory::MemoryVault,
    provider::ProviderKind,
    storage::{
        audit_path, cache_path, incidents_path, integrations_path, kill_switch_path, load_audit,
        load_cache, load_incidents, load_integrations, load_kill_switch, load_notifications,
        load_swarm_events, notifications_path, save_integrations, save_kill_switch,
        swarm_events_path,
    },
};

#[derive(Debug, Clone, Serialize)]
pub struct StatusSnapshot {
    pub provider: ProviderKind,
    pub dry_run: bool,
    pub cache_entries: usize,
    pub memory_entries: usize,
    pub kill_switch: bool,
}

#[derive(Clone)]
pub struct SharedState {
    pub status: Arc<Mutex<StatusSnapshot>>,
}

impl SharedState {
    pub fn new(status: StatusSnapshot) -> Self {
        Self {
            status: Arc::new(Mutex::new(status)),
        }
    }

    pub fn update(&self, cache: &CacheState, memory: &MemoryVault) {
        if let Ok(mut status) = self.status.lock() {
            status.cache_entries = cache.files.len();
            status.memory_entries = memory.entries.len();
        }
    }
}

pub fn serve(state: SharedState, addr: &str) -> anyhow::Result<()> {
    let server = Server::http(addr).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    println!("Nexus interface listening on http://{}", addr);

    for request in server.incoming_requests() {
        let method = request.method();
        let url = request.url();

        let response = match (method, url) {
            (&Method::Get, "/") => html_response(dashboard_html()),
            (&Method::Get, "/app.js") => js_response(app_js()),
            (&Method::Get, "/style.css") => css_response(app_css()),
            (&Method::Get, "/manifest.json") => json_response(manifest_json())?,
            (&Method::Get, "/sw.js") => js_response(service_worker_js()),
            (&Method::Get, "/health") => Response::from_string("ok"),
            (&Method::Get, "/status") => {
                let mut snapshot = state.status.lock().unwrap().clone();
                snapshot.kill_switch = load_kill_switch(&kill_switch_path()?).unwrap_or(false);
                let body = serde_json::to_string_pretty(&snapshot)?;
                json_response(body)?
            }
            (&Method::Get, "/incidents") => {
                let incidents = load_incidents(&incidents_path()?)?;
                let body = serde_json::to_string_pretty(&incidents)?;
                json_response(body)?
            }
            (&Method::Get, "/audit") => {
                let audit = load_audit(&audit_path()?)?;
                let body = serde_json::to_string_pretty(&audit)?;
                json_response(body)?
            }
            (&Method::Get, "/diff") => {
                let cached = load_cache(&cache_path()?)?;
                let root = if cached.root.as_os_str().is_empty() {
                    std::path::PathBuf::from(".")
                } else {
                    cached.root.clone()
                };
                let mut current = CacheState::new(root);
                let _ = current.warm();
                let diff = cached.diff(&current);
                let body = serde_json::to_string_pretty(&diff)?;
                json_response(body)?
            }
            (&Method::Get, "/notifications") => {
                let notifications = load_notifications(&notifications_path()?)?;
                let body = serde_json::to_string_pretty(&notifications)?;
                json_response(body)?
            }
            (&Method::Get, "/swarm-events") => {
                let events = load_swarm_events(&swarm_events_path()?)?;
                let body = serde_json::to_string_pretty(&events)?;
                json_response(body)?
            }
            (&Method::Get, "/integrations") => {
                let integrations = load_integrations(&integrations_path()?)?;
                let body = serde_json::to_string_pretty(&integrations)?;
                json_response(body)?
            }
            (&Method::Post, path) if path.starts_with("/integrations/enable") => {
                let name = query_param(path, "name");
                if let Some(name) = name {
                    let path = integrations_path()?;
                    let mut integrations = load_integrations(&path)?;
                    if crate::mcp::set_enabled(&mut integrations, &name, true) {
                        save_integrations(&integrations, &path)?;
                        Response::from_string("ok")
                    } else {
                        Response::from_string("unknown integration").with_status_code(404)
                    }
                } else {
                    Response::from_string("missing name").with_status_code(400)
                }
            }
            (&Method::Post, path) if path.starts_with("/integrations/disable") => {
                let name = query_param(path, "name");
                if let Some(name) = name {
                    let path = integrations_path()?;
                    let mut integrations = load_integrations(&path)?;
                    if crate::mcp::set_enabled(&mut integrations, &name, false) {
                        save_integrations(&integrations, &path)?;
                        Response::from_string("ok")
                    } else {
                        Response::from_string("unknown integration").with_status_code(404)
                    }
                } else {
                    Response::from_string("missing name").with_status_code(400)
                }
            }
            (&Method::Post, "/kill-switch/on") => {
                let path = kill_switch_path()?;
                save_kill_switch(true, &path)?;
                if let Ok(mut status) = state.status.lock() {
                    status.kill_switch = true;
                }
                Response::from_string("ok")
            }
            (&Method::Post, "/kill-switch/off") => {
                let path = kill_switch_path()?;
                save_kill_switch(false, &path)?;
                if let Ok(mut status) = state.status.lock() {
                    status.kill_switch = false;
                }
                Response::from_string("ok")
            }
            (&Method::Get, "/kill-switch") => {
                let enabled = load_kill_switch(&kill_switch_path()?)?;
                let body = serde_json::to_string_pretty(&enabled)?;
                json_response(body)?
            }
            _ => Response::from_string("not found").with_status_code(404),
        };

        let _ = request.respond(response);
    }

    Ok(())
}

fn query_param(url: &str, key: &str) -> Option<String> {
    let query = url.splitn(2, '?').nth(1)?;
    for pair in query.split('&') {
        let mut iter = pair.splitn(2, '=');
        if let (Some(k), Some(v)) = (iter.next(), iter.next()) {
            if k == key {
                return Some(url_decode(v));
            }
        }
    }
    None
}

fn url_decode(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '%' => {
                let first = chars.next();
                let second = chars.next();
                if let (Some(first), Some(second)) = (first, second) {
                    if let Ok(byte) = u8::from_str_radix(&format!("{}{}", first, second), 16) {
                        output.push(byte as char);
                    }
                }
            }
            '+' => output.push(' '),
            _ => output.push(ch),
        }
    }
    output
}

fn json_response(body: String) -> anyhow::Result<Response<std::io::Cursor<Vec<u8>>>> {
    let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
        .map_err(|_| anyhow::anyhow!("Invalid header"))?;
    Ok(Response::from_string(body).with_header(header))
}

fn html_response(body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
        .unwrap();
    Response::from_string(body).with_header(header)
}

fn js_response(body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let header = Header::from_bytes(&b"Content-Type"[..], &b"application/javascript"[..])
        .unwrap();
    Response::from_string(body).with_header(header)
}

fn css_response(body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let header = Header::from_bytes(&b"Content-Type"[..], &b"text/css"[..]).unwrap();
    Response::from_string(body).with_header(header)
}

fn dashboard_html() -> &'static str {
    r##"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="theme-color" content="#0f1217" />
    <title>Nexus Dashboard</title>
    <link rel="stylesheet" href="/style.css" />
    <link rel="manifest" href="/manifest.json" />
  </head>
  <body>
    <main>
      <header>
        <div>
          <h1>Nexus Dashboard</h1>
          <p>Live status, swarm activity, and safety controls.</p>
        </div>
        <button id="kill-switch" class="danger">Kill Switch</button>
      </header>

      <section class="grid">
        <article>
          <h2>Status</h2>
          <div id="status"></div>
        </article>
        <article>
          <h2>Audit</h2>
          <div id="audit"></div>
        </article>
        <article>
          <h2>Notifications</h2>
          <div id="notifications"></div>
        </article>
        <article>
          <h2>Incidents</h2>
          <div id="incidents"></div>
        </article>
        <article>
          <h2>Diff Review</h2>
          <div id="diff"></div>
        </article>
      </section>

      <section class="panel">
        <h2>MCP Marketplace</h2>
        <p>Configure integrations and external tools from this hub.</p>
        <div id="integrations" class="pill-row"></div>
      </section>

      <section class="panel">
        <h2>Swarm Activity</h2>
        <p>Awaiting live agent events. Connect CLI swarm runs to populate.</p>
        <div id="swarm"></div>
      </section>
    </main>
    <script src="/app.js"></script>
  </body>
</html>
"##
}

fn app_js() -> &'static str {
    r#"const statusEl = document.getElementById("status");
const auditEl = document.getElementById("audit");
const notificationsEl = document.getElementById("notifications");
const incidentsEl = document.getElementById("incidents");
const diffEl = document.getElementById("diff");
const killSwitch = document.getElementById("kill-switch");
const integrationsEl = document.getElementById("integrations");

async function fetchJson(path) {
  const res = await fetch(path);
  if (!res.ok) {
    throw new Error(`Request failed: ${path}`);
  }
  return res.json();
}

if ("serviceWorker" in navigator) {
  navigator.serviceWorker.register("/sw.js").catch(() => {});
}

function renderList(target, items, emptyMessage) {
  if (!items || items.length === 0) {
    target.innerHTML = `<p class="muted">${emptyMessage}</p>`;
    return;
  }
  target.innerHTML = `<ul>${items.map((item) => `<li>${item}</li>`).join("")}</ul>`;
}

function renderIntegrations(items) {
  if (!items || items.length === 0) {
    integrationsEl.innerHTML = `<p class="muted">No integrations configured.</p>`;
    return;
  }
  integrationsEl.innerHTML = items
    .map(
      (integration) =>
        `<div class="integration ${integration.enabled ? "enabled" : ""}">
          <div>
            <strong>${integration.name}</strong>
            <div class="muted">${integration.kind}</div>
          </div>
          <button data-name="${integration.name}" class="toggle">
            ${integration.enabled ? "Disable" : "Enable"}
          </button>
        </div>`
    )
    .join("");
  integrationsEl.querySelectorAll("button.toggle").forEach((button) => {
    button.addEventListener("click", async () => {
      const name = button.getAttribute("data-name");
      const enabled = button.closest(".integration").classList.contains("enabled");
      await fetch(
        enabled
          ? `/integrations/disable?name=${encodeURIComponent(name)}`
          : `/integrations/enable?name=${encodeURIComponent(name)}`,
        { method: "POST" }
      );
      refresh();
    });
  });
}

async function refresh() {
  try {
    const [status, audit, notifications, incidents, diff, kill, integrations, swarm] = await Promise.all([
      fetchJson("/status"),
      fetchJson("/audit"),
      fetchJson("/notifications"),
      fetchJson("/incidents"),
      fetchJson("/diff"),
      fetchJson("/kill-switch"),
      fetchJson("/integrations"),
      fetchJson("/swarm-events"),
    ]);

    statusEl.innerHTML = `
      <div><strong>Provider:</strong> ${status.provider}</div>
      <div><strong>Dry run:</strong> ${status.dry_run}</div>
      <div><strong>Cache entries:</strong> ${status.cache_entries}</div>
      <div><strong>Memory entries:</strong> ${status.memory_entries}</div>
    `;

    auditEl.innerHTML = `
      <div>Performance benchmark: ${audit.performance_benchmark ? "✅" : "—"}</div>
      <div>Security audit: ${audit.security_audit ? "✅" : "—"}</div>
      <div>Docs complete: ${audit.docs_complete ? "✅" : "—"}</div>
    `;

    renderList(
      notificationsEl,
      notifications.map((item) => `[${item.level}] ${item.message}`),
      "No notifications yet."
    );

    renderList(
      incidentsEl,
      incidents.map((incident) => `[${incident.kind}] ${incident.summary}`),
      "No incidents detected."
    );

    renderList(
      diffEl,
      diff.changed.map((item) => `Changed: ${item}`).concat(diff.removed.map((item) => `Removed: ${item}`)),
      "No pending diffs."
    );

    killSwitch.classList.toggle("armed", kill);
    killSwitch.textContent = kill ? "Kill Switch Armed" : "Kill Switch";

    renderIntegrations(integrations);
    renderList(
      document.getElementById("swarm"),
      swarm.map((item) => `[${item.event}] ${item.detail}`),
      "Awaiting swarm activity."
    );
  } catch (err) {
    statusEl.innerHTML = `<p class="muted">Failed to load status.</p>`;
  }
}

killSwitch.addEventListener("click", async () => {
  const armed = killSwitch.classList.contains("armed");
  await fetch(armed ? "/kill-switch/off" : "/kill-switch/on", { method: "POST" });
  refresh();
});

refresh();
setInterval(refresh, 4000);
"#
}

fn app_css() -> &'static str {
    r#"
    :root {
      color-scheme: dark;
      --bg: #0f1217;
      --panel: #171c24;
      --text: #f4f6fb;
      --muted: #9aa4b2;
      --accent: #6cc1ff;
      --danger: #ff5f6d;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      background: var(--bg);
      color: var(--text);
      font-family: "Inter", system-ui, sans-serif;
    }
    main { padding: 32px; max-width: 1200px; margin: 0 auto; }
    header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 16px;
      margin-bottom: 24px;
    }
    h1 { margin: 0; font-size: 32px; }
    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
      gap: 16px;
      margin-bottom: 24px;
    }
    article, .panel {
      background: var(--panel);
      border-radius: 12px;
      padding: 16px;
      box-shadow: 0 12px 24px rgba(0, 0, 0, 0.2);
    }
    .panel { margin-bottom: 24px; }
    .pill-row { display: flex; gap: 8px; flex-wrap: wrap; }
    .pill {
      background: rgba(108, 193, 255, 0.2);
      padding: 6px 12px;
      border-radius: 999px;
      font-size: 12px;
    }
    .integration {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
      padding: 8px 12px;
      border-radius: 10px;
      background: rgba(108, 193, 255, 0.08);
      flex: 1 1 220px;
    }
    .integration.enabled {
      background: rgba(46, 204, 113, 0.2);
    }
    .integration .toggle {
      background: var(--accent);
      color: #0f1217;
    }
    button {
      background: var(--danger);
      border: none;
      color: white;
      padding: 10px 16px;
      border-radius: 999px;
      cursor: pointer;
      font-weight: 600;
    }
    button.armed {
      background: #2ecc71;
    }
    .muted { color: var(--muted); }
    ul { padding-left: 16px; margin: 8px 0 0; }
    li { margin-bottom: 4px; }
    "#
}

fn manifest_json() -> String {
    serde_json::json!({
        "name": "Nexus Dashboard",
        "short_name": "Nexus",
        "start_url": "/",
        "display": "standalone",
        "background_color": "#0f1217",
        "theme_color": "#0f1217",
        "icons": []
    })
    .to_string()
}

fn service_worker_js() -> &'static str {
    r#"
    const CACHE = "nexus-dashboard-v1";
    const ASSETS = ["/", "/style.css", "/app.js", "/manifest.json"];

    self.addEventListener("install", (event) => {
      event.waitUntil(
        caches.open(CACHE).then((cache) => cache.addAll(ASSETS)).then(() => self.skipWaiting())
      );
    });

    self.addEventListener("activate", (event) => {
      event.waitUntil(
        caches.keys().then((keys) =>
          Promise.all(keys.map((key) => (key === CACHE ? null : caches.delete(key))))
        ).then(() => self.clients.claim())
      );
    });

    self.addEventListener("fetch", (event) => {
      const { request } = event;
      if (request.method !== "GET") {
        return;
      }
      event.respondWith(
        caches.match(request).then((cached) =>
          cached || fetch(request).then((response) => response)
        )
      );
    });
    "#
}
