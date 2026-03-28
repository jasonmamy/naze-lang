use axum::{response::Html, routing::get, Router};
use std::sync::Arc;

use crate::api::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(overview))
        .route("/ui/services", get(services_page))
        .route("/ui/trust", get(trust_page))
        .route("/ui/patterns", get(patterns_page))
        .route("/ui/observations", get(observations_page))
        .route("/ui/test", get(test_console))
}

async fn overview() -> Html<String> {
    Html(page(
        "Overview",
        r#"
        <div class="stats" id="stats"></div>
        <h2>Recent Activity</h2>
        <div id="activity">Loading...</div>
        <script>
        async function load() {
            const info = await (await fetch('/api/v1/discovery/info')).json();
            document.getElementById('stats').innerHTML = `
                <div class="stat"><div class="value">${info.services}</div><div class="label">Services</div></div>
                <div class="stat"><div class="value">${info.peers}</div><div class="label">Peers</div></div>
                <div class="stat"><div class="value">${info.profiles.length}</div><div class="label">Trust Profiles</div></div>
            `;
            document.getElementById('activity').innerHTML = `
                <p>Network: <strong>${info.network_id}</strong> (${info.scope}) | Version: ${info.version}</p>
                <p>Profiles: ${info.profiles.join(', ')}</p>
            `;
        }
        load(); setInterval(load, 5000);
        </script>
        "#,
    ))
}

async fn services_page() -> Html<String> {
    Html(page(
        "Services",
        r#"
        <div id="services">Loading...</div>
        <script>
        async function load() {
            const r = await fetch('/api/v1/discovery/export');
            const services = await r.json();
            if (!services.length) { document.getElementById('services').innerHTML = '<p>No services registered.</p>'; return; }
            let html = '<table><tr><th>Domain</th><th>Name</th><th>Version</th><th>Publisher</th></tr>';
            for (const s of services) {
                html += `<tr>
                    <td>${s.domain}</td>
                    <td><a href="/api/v1/discovery/services/${s.domain}/${encodeURIComponent(s.name)}">${s.name}</a></td>
                    <td>${s.version}</td>
                    <td>${s.publisher || '-'}</td>
                </tr>`;
            }
            html += '</table>';
            document.getElementById('services').innerHTML = html;
        }
        load();
        </script>
        "#,
    ))
}

async fn trust_page() -> Html<String> {
    Html(page(
        "Trust Dynamics",
        r#"
        <div id="trust">Loading...</div>
        <script>
        async function load() {
            const services = await (await fetch('/api/v1/discovery/export')).json();
            let html = '<table><tr><th>Service</th><th>Domain</th>';
            // Get profile names
            const profiles = await (await fetch('/api/v1/discovery/profiles')).json();
            for (const p of profiles) html += `<th>${p.name}</th>`;
            html += '</tr>';
            for (const s of services) {
                const trust = await (await fetch(`/api/v1/discovery/services/${s.domain}/${encodeURIComponent(s.name)}/trust`)).json();
                html += `<tr><td>${s.name}</td><td>${s.domain}</td>`;
                for (const p of profiles) {
                    const score = trust.scores?.[p.name]?.score;
                    const color = score >= 0.8 ? '#2d7' : score >= 0.5 ? '#da3' : '#d33';
                    html += `<td style="color:${color};font-weight:bold">${score?.toFixed(2) ?? '-'}</td>`;
                }
                html += '</tr>';
            }
            html += '</table>';
            document.getElementById('trust').innerHTML = html;
        }
        load();
        </script>
        "#,
    ))
}

async fn patterns_page() -> Html<String> {
    Html(page(
        "Composition Patterns",
        r#"
        <div id="patterns">Loading...</div>
        <script>
        async function load() {
            const data = await (await fetch('/api/v1/discovery/patterns?limit=20')).json();
            if (!data.patterns.length) { document.getElementById('patterns').innerHTML = '<p>No patterns yet.</p>'; return; }
            let html = '<table><tr><th>Services</th><th>Frequency</th><th>First Seen</th><th>Last Seen</th></tr>';
            for (const p of data.patterns) {
                const names = p.services.map(s => `${s.name} (${s.domain})`).join(' + ');
                html += `<tr><td>${names}</td><td><strong>${p.frequency}</strong></td><td>${p.first_seen}</td><td>${p.last_seen}</td></tr>`;
            }
            html += '</table>';
            document.getElementById('patterns').innerHTML = html;
        }
        load();
        </script>
        "#,
    ))
}

async fn observations_page() -> Html<String> {
    Html(page(
        "Observations",
        r#"
        <p>View observation signals for a specific service:</p>
        <form id="obs-form" onsubmit="return loadSignals()">
            <input id="obs-domain" placeholder="domain" value="bakery.example.com" />
            <input id="obs-name" placeholder="name" value="SweetCakes" />
            <button type="submit">Load Signals</button>
        </form>
        <div id="signals"></div>
        <script>
        async function loadSignals() {
            const d = document.getElementById('obs-domain').value;
            const n = document.getElementById('obs-name').value;
            const r = await fetch(`/api/v1/discovery/services/${d}/${encodeURIComponent(n)}/signals`);
            const data = await r.json();
            document.getElementById('signals').innerHTML = `<pre>${JSON.stringify(data, null, 2)}</pre>`;
            return false;
        }
        </script>
        "#,
    ))
}

async fn test_console() -> Html<String> {
    Html(page(
        "Test Console",
        r#"
        <h2>Register a Service</h2>
        <textarea id="reg-body" rows="12" style="width:100%;font-family:monospace;font-size:0.85rem">{
  "domain": "test.local",
  "manifest": {
    "name": "TestService",
    "state": {"counter": {"type": "number"}},
    "server_functions": ["increment"],
    "actions": ["click"]
  }
}</textarea>
        <button onclick="doRegister()">Register</button>
        <div id="reg-result"></div>

        <h2>Search by Capability</h2>
        <textarea id="search-body" rows="6" style="width:100%;font-family:monospace;font-size:0.85rem">{
  "require": [{"kind": "server_function", "name": "order"}]
}</textarea>
        <button onclick="doSearch()">Search</button>
        <div id="search-result"></div>

        <h2>Flag a Service</h2>
        <input id="flag-domain" placeholder="domain" />
        <input id="flag-name" placeholder="name" />
        <input id="flag-reason" placeholder="reason" />
        <button onclick="doFlag()">Flag</button>
        <div id="flag-result"></div>

        <script>
        async function doRegister() {
            try {
                const body = document.getElementById('reg-body').value;
                const r = await fetch('/api/v1/discovery/services', {method:'POST', headers:{'content-type':'application/json'}, body});
                const data = await r.json();
                document.getElementById('reg-result').innerHTML = `<pre>${JSON.stringify(data, null, 2)}</pre>`;
            } catch(e) { document.getElementById('reg-result').textContent = 'Error: ' + e; }
        }
        async function doSearch() {
            try {
                const body = document.getElementById('search-body').value;
                const r = await fetch('/api/v1/discovery/search', {method:'POST', headers:{'content-type':'application/json'}, body});
                const data = await r.json();
                document.getElementById('search-result').innerHTML = `<pre>${JSON.stringify(data, null, 2)}</pre>`;
            } catch(e) { document.getElementById('search-result').textContent = 'Error: ' + e; }
        }
        async function doFlag() {
            try {
                const body = JSON.stringify({
                    service_domain: document.getElementById('flag-domain').value,
                    service_name: document.getElementById('flag-name').value,
                    reason: document.getElementById('flag-reason').value
                });
                const r = await fetch('/api/v1/discovery/flag', {method:'POST', headers:{'content-type':'application/json'}, body});
                const data = await r.json();
                document.getElementById('flag-result').innerHTML = `<pre>${JSON.stringify(data, null, 2)}</pre>`;
            } catch(e) { document.getElementById('flag-result').textContent = 'Error: ' + e; }
        }
        </script>
        "#,
    ))
}

fn page(title: &str, content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Naze Discovery — {title}</title>
    <style>
        body {{ font-family: system-ui, sans-serif; max-width: 960px; margin: 2rem auto; padding: 0 1rem; color: #333; }}
        h1 {{ color: #1a1a2e; margin-bottom: 0.5rem; }}
        nav {{ margin-bottom: 1.5rem; padding: 0.75rem 0; border-bottom: 1px solid #ddd; }}
        nav a {{ margin-right: 1rem; text-decoration: none; color: #1a1a2e; font-weight: 500; }}
        nav a:hover {{ text-decoration: underline; }}
        .stats {{ display: flex; gap: 1rem; margin: 1rem 0; }}
        .stat {{ background: #f5f5f5; padding: 1rem; border-radius: 8px; flex: 1; text-align: center; }}
        .stat .value {{ font-size: 2rem; font-weight: bold; color: #1a1a2e; }}
        .stat .label {{ color: #666; font-size: 0.875rem; }}
        table {{ width: 100%; border-collapse: collapse; margin: 1rem 0; }}
        th, td {{ padding: 0.5rem; border-bottom: 1px solid #eee; text-align: left; }}
        th {{ background: #f5f5f5; font-weight: 600; }}
        pre {{ background: #f5f5f5; padding: 1rem; border-radius: 8px; overflow-x: auto; font-size: 0.85rem; }}
        button {{ background: #1a1a2e; color: white; border: none; padding: 0.5rem 1rem; border-radius: 4px; cursor: pointer; margin: 0.5rem 0; }}
        button:hover {{ background: #2a2a4e; }}
        input, textarea {{ border: 1px solid #ddd; padding: 0.5rem; border-radius: 4px; margin: 0.25rem; }}
    </style>
</head>
<body>
    <h1>Naze Discovery Network</h1>
    <nav>
        <a href="/">Overview</a>
        <a href="/ui/services">Services</a>
        <a href="/ui/trust">Trust</a>
        <a href="/ui/patterns">Patterns</a>
        <a href="/ui/observations">Observations</a>
        <a href="/ui/test">Test Console</a>
    </nav>
    <h2>{title}</h2>
    {content}
</body>
</html>"#,
        title = title,
        content = content,
    )
}
