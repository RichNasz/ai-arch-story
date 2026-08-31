use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use super::{routes::api_router, state::AppState};

static NEXT_WORKSPACE: AtomicUsize = AtomicUsize::new(0);

struct TestWorkspace {
    root: PathBuf,
    parent: PathBuf,
}

impl TestWorkspace {
    fn new() -> Self {
        let nonce = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
        let parent = std::env::temp_dir().join(format!(
            "ai-arch-story-route-tests-{}-{}",
            std::process::id(),
            nonce
        ));
        let root = parent.join("workspace");
        fs::create_dir_all(&root).expect("create isolated test workspace");
        Self { root, parent }
    }

    fn app(&self) -> Router {
        api_router().with_state(AppState::new(self.root.clone()))
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.parent);
    }
}

async fn request(app: Router, method: &str, uri: String, body: Body) -> (StatusCode, String) {
    let response = app
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("content-type", "application/json")
                .body(body)
                .expect("build route request"),
        )
        .await
        .expect("router response");
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read route response body");
    (
        status,
        String::from_utf8(body.to_vec()).expect("response is UTF-8"),
    )
}

async fn assert_invalid_name(workspace: &TestWorkspace, method: &str, uri: String, body: Body) {
    let (status, response) = request(workspace.app(), method, uri, body).await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "{method} must reject invalid diagram names"
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&response).expect("error response is JSON")["error"]
            ["code"],
        "INVALID_DIAGRAM_NAME",
        "{method} must use the diagram-name contract error code"
    );
    assert_workspace_has_no_diagrams(&workspace.root);
}

fn assert_workspace_has_no_diagrams(workspace: &Path) {
    let diagrams = workspace.join("diagrams");
    if diagrams.exists() {
        assert!(
            fs::read_dir(&diagrams)
                .expect("read diagrams directory")
                .next()
                .is_none(),
            "invalid diagram names must not create workspace files"
        );
    }
    assert!(
        !workspace.join("escape").exists(),
        "traversal must not create files outside the diagrams directory"
    );
}

fn route_segment(name: &str) -> String {
    name.replace('%', "%25")
        .replace('/', "%2F")
        .replace('\\', "%5C")
}

#[tokio::test]
async fn diagram_routes_reject_unsafe_slugs_without_creating_files() {
    // Removing slug validation from any route must make this test fail.
    let workspace = TestWorkspace::new();
    let invalid_names = [
        "../escape",
        "/tmp/x",
        "a/b",
        "a\\b",
        ".",
        "..",
        "System-Overview",
    ];

    for name in invalid_names {
        let encoded_name = route_segment(name);
        let create_body = serde_json::json!({ "name": name, "title": "Unsafe" }).to_string();
        assert_invalid_name(
            &workspace,
            "POST",
            "/diagrams".to_string(),
            Body::from(create_body),
        )
        .await;
        assert_invalid_name(
            &workspace,
            "GET",
            format!("/diagrams/{encoded_name}"),
            Body::empty(),
        )
        .await;
        assert_invalid_name(
            &workspace,
            "POST",
            format!("/diagrams/{encoded_name}/render"),
            Body::empty(),
        )
        .await;
        assert_invalid_name(
            &workspace,
            "GET",
            format!("/diagrams/{encoded_name}/preview"),
            Body::empty(),
        )
        .await;
        assert_invalid_name(
            &workspace,
            "DELETE",
            format!("/diagrams/{encoded_name}"),
            Body::empty(),
        )
        .await;
    }
}

#[tokio::test]
async fn valid_system_overview_slug_creates_a_diagram() {
    // Rejecting a compliant slug must make this test fail.
    let workspace = TestWorkspace::new();
    let (status, response) = request(
        workspace.app(),
        "POST",
        "/diagrams".to_string(),
        Body::from(
            serde_json::json!({ "name": "system-overview", "title": "System Overview" })
                .to_string(),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&response).expect("create response is JSON")["title"],
        "System Overview"
    );
    assert!(
        workspace
            .root
            .join("diagrams/system-overview/diagram.json")
            .is_file()
    );
}
