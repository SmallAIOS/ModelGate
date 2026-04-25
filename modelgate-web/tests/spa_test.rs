//! Integration tests for the static SPA surface.
//!
//! The modelgate-web crate embeds `ui/modelgate-web/dist/` at compile
//! time. These tests assert that `/` returns `index.html`, that built
//! assets are served with appropriate content-types, and that unknown
//! non-API paths fall through to `index.html` (so the SPA hash router
//! can handle client-side routing).

use smctl_gate::{GateClient, GateConfig};
use wiremock::MockServer;

async fn spawn_server(upstream_url: String) -> String {
    let client = GateClient::new(GateConfig {
        url: upstream_url,
        timeout_secs: 5,
    })
    .unwrap();
    let app = modelgate_web::router(client);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

#[tokio::test]
async fn root_returns_embedded_index_html() {
    let upstream = MockServer::start().await;
    let base = spawn_server(upstream.uri()).await;

    let resp = reqwest::get(format!("{base}/")).await.unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        ct.starts_with("text/html"),
        "content-type should be HTML, got {ct}"
    );
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("<div id=\"root\"></div>"),
        "index.html should carry the React mount point"
    );
    assert!(
        body.contains("ModelGate"),
        "index.html title should mention ModelGate"
    );
}

#[tokio::test]
async fn unknown_path_falls_back_to_index_html() {
    // The SPA uses hash routing, so deep links are `/#/models` — but a
    // user who reloads an accidentally-generated path like `/anything`
    // should still see the app shell rather than a blank 404.
    let upstream = MockServer::start().await;
    let base = spawn_server(upstream.uri()).await;

    let resp = reqwest::get(format!("{base}/deep-link-that-does-not-exist"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(
        body.contains("<div id=\"root\"></div>"),
        "fallback should serve index.html"
    );
}

#[tokio::test]
async fn api_paths_do_not_leak_to_spa_fallback() {
    // An unmatched /api/* request must NOT return index.html — that
    // would break JSON clients that check content-type on failure.
    let upstream = MockServer::start().await;
    let base = spawn_server(upstream.uri()).await;

    let resp = reqwest::get(format!("{base}/api/not-a-real-route"))
        .await
        .unwrap();
    // axum returns 404 for unrouted paths; we just need to verify it's
    // NOT the HTML shell.
    let ct = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        !ct.starts_with("text/html"),
        "unmatched /api/* must not serve the SPA shell, got content-type {ct}"
    );
}
