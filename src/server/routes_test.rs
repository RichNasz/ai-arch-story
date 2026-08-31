use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use super::{
    routes::api_router,
    state::{AppState, AtomicJsonWriteError, atomic_write_json},
};

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
    request_with_content_type(app, method, uri, "application/json", body).await
}

async fn request_with_content_type(
    app: Router,
    method: &str,
    uri: String,
    content_type: &str,
    body: Body,
) -> (StatusCode, String) {
    let response = app
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("content-type", content_type)
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

fn multipart_shape_body(
    boundary: &str,
    name: &str,
    file_name: &str,
    content_type: &str,
    content: &str,
) -> String {
    format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"name\"\r\n\r\n\
         {name}\r\n\
         --{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\n\
         Content-Type: {content_type}\r\n\r\n\
         {content}\r\n\
         --{boundary}--\r\n"
    )
}

#[tokio::test]
async fn shared_theme_round_trips_an_object_and_persists_it() {
    // Removing either theme route or its persisted write must make this test fail.
    let workspace = TestWorkspace::new();
    let theme = serde_json::json!({
        "light": { "background": "#ffffff", "text": "#111827" },
        "dark": { "background": "#111827", "text": "#ffffff" }
    });

    let (put_status, put_response) = request(
        workspace.app(),
        "PUT",
        "/shared/theme".to_string(),
        Body::from(theme.to_string()),
    )
    .await;
    assert_eq!(put_status, StatusCode::OK);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&put_response).expect("PUT response is JSON"),
        theme
    );

    let (get_status, get_response) = request(
        workspace.app(),
        "GET",
        "/shared/theme".to_string(),
        Body::empty(),
    )
    .await;
    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&get_response).expect("GET response is JSON"),
        theme
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(workspace.root.join("shared/theme.json"))
                .expect("read persisted theme")
        )
        .expect("persisted theme is JSON"),
        theme
    );
}

#[tokio::test]
async fn shared_theme_rejects_a_non_object_without_overwriting_the_file() {
    // Accepting a non-object theme or changing the prior file must make this test fail.
    let workspace = TestWorkspace::new();
    let shared = workspace.root.join("shared");
    fs::create_dir_all(&shared).expect("create shared directory");
    let original = serde_json::json!({ "light": { "background": "#ffffff" } });
    fs::write(shared.join("theme.json"), original.to_string()).expect("seed theme");

    let (status, response) = request(
        workspace.app(),
        "PUT",
        "/shared/theme".to_string(),
        Body::from("[]"),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&response).expect("error response is JSON")["error"]
            ["code"],
        "INVALID_THEME"
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(shared.join("theme.json")).expect("read unchanged theme")
        )
        .expect("stored theme is JSON"),
        original
    );
}

#[tokio::test]
async fn project_shape_upload_accepts_a_multipart_svg_file() {
    // Reverting this endpoint to JSON or failing to persist the file must make this test fail.
    let workspace = TestWorkspace::new();
    let boundary = "shape-upload-boundary";
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><path d="M0 0h100v100H0z" fill="currentColor"/></svg>"#;
    let body = multipart_shape_body(
        boundary,
        "cloud-native",
        "cloud-native.svg",
        "image/svg+xml",
        svg,
    );

    let (status, response) = request_with_content_type(
        workspace.app(),
        "POST",
        "/project/shapes".to_string(),
        &format!("multipart/form-data; boundary={boundary}"),
        Body::from(body),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&response).expect("upload response is JSON"),
        serde_json::json!({ "name": "cloud-native" })
    );
    assert_eq!(
        fs::read_to_string(workspace.root.join("shared/shapes/cloud-native.svg"))
            .expect("read uploaded shape"),
        svg
    );

    let (list_status, list_response) = request(
        workspace.app(),
        "GET",
        "/project/shapes".to_string(),
        Body::empty(),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&list_response)
            .expect("shape list response is JSON"),
        serde_json::json!({ "shapes": [{ "name": "cloud-native" }] })
    );
}

#[tokio::test]
async fn project_shape_upload_rejects_non_svg_content_without_writing_a_file() {
    // Trusting only the multipart filename or media type must make this test fail.
    let workspace = TestWorkspace::new();
    let boundary = "invalid-shape-boundary";
    let body = multipart_shape_body(
        boundary,
        "not-a-shape",
        "not-a-shape.svg",
        "image/svg+xml",
        "<html>not an SVG</html>",
    );

    let (status, response) = request_with_content_type(
        workspace.app(),
        "POST",
        "/project/shapes".to_string(),
        &format!("multipart/form-data; boundary={boundary}"),
        Body::from(body),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&response).expect("error response is JSON")["error"]
            ["code"],
        "INVALID_SVG"
    );
    assert!(!workspace.root.join("shared/shapes/not-a-shape.svg").exists());
}

#[tokio::test]
async fn project_shape_upload_rejects_svg_without_a_view_box() {
    // Accepting an SVG that the renderer cannot scale must make this test fail.
    let workspace = TestWorkspace::new();
    let boundary = "missing-view-box-boundary";
    let body = multipart_shape_body(
        boundary,
        "unscalable",
        "unscalable.svg",
        "image/svg+xml",
        "<svg><path d=\"M0 0h10v10H0z\"/></svg>",
    );

    let (status, response) = request_with_content_type(
        workspace.app(),
        "POST",
        "/project/shapes".to_string(),
        &format!("multipart/form-data; boundary={boundary}"),
        Body::from(body),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&response).expect("error response is JSON")["error"]
            ["code"],
        "INVALID_SVG"
    );
    assert!(!workspace.root.join("shared/shapes/unscalable.svg").exists());
}

#[tokio::test]
async fn project_shape_upload_rejects_active_or_external_svg_content() {
    // Persisting executable or externally referenced SVG markup must make this test fail.
    let workspace = TestWorkspace::new();

    for (index, svg) in [
        r#"<svg viewBox="0 0 10 10"><script>alert(1)</script></svg>"#,
        r#"<svg viewBox="0 0 10 10" onload="alert(1)"><path d="M0 0h10v10H0z"/></svg>"#,
        r#"<svg viewBox="0 0 10 10"><use href="https://example.com/shape.svg#x"/></svg>"#,
    ]
    .into_iter()
    .enumerate()
    {
        let boundary = format!("active-svg-boundary-{index}");
        let body = multipart_shape_body(
            &boundary,
            &format!("unsafe-content-{index}"),
            "unsafe.svg",
            "image/svg+xml",
            svg,
        );
        let (status, response) = request_with_content_type(
            workspace.app(),
            "POST",
            "/project/shapes".to_string(),
            &format!("multipart/form-data; boundary={boundary}"),
            Body::from(body),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&response)
                .expect("error response is JSON")["error"]["code"],
            "INVALID_SVG"
        );
    }
}

#[tokio::test]
async fn project_shape_upload_returns_json_for_malformed_multipart() {
    // Letting extractor failures bypass the API error envelope must make this test fail.
    let workspace = TestWorkspace::new();
    let (status, response) = request_with_content_type(
        workspace.app(),
        "POST",
        "/project/shapes".to_string(),
        "multipart/form-data",
        Body::from("not multipart"),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&response).expect("error response is JSON")["error"]
            ["code"],
        "INVALID_MULTIPART"
    );
}

#[tokio::test]
async fn project_shape_upload_rejects_unsafe_names_without_writing_files() {
    // Allowing any name outside the kebab-case asset contract must make this test fail.
    let workspace = TestWorkspace::new();
    let svg = r#"<svg viewBox="0 0 10 10"><path d="M0 0h10v10H0z"/></svg>"#;

    for (index, name) in [
        "../escape",
        "nested/shape",
        r"nested\shape",
        "shape.svg",
        "Uppercase",
        "two--hyphens",
        "-leading",
        "trailing-",
    ]
    .into_iter()
    .enumerate()
    {
        let boundary = format!("unsafe-shape-boundary-{index}");
        let body = multipart_shape_body(&boundary, name, "shape.svg", "image/svg+xml", svg);
        let (status, response) = request_with_content_type(
            workspace.app(),
            "POST",
            "/project/shapes".to_string(),
            &format!("multipart/form-data; boundary={boundary}"),
            Body::from(body),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "must reject {name:?}");
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&response)
                .expect("error response is JSON")["error"]["code"],
            "INVALID_NAME",
            "must use the shape-name error contract for {name:?}"
        );
    }

    assert!(!workspace.root.join("escape.svg").exists());
    let shapes = workspace.root.join("shared/shapes");
    assert!(
        !shapes.exists()
            || fs::read_dir(shapes)
                .expect("read shapes directory")
                .next()
                .is_none(),
        "unsafe uploads must not create shape files"
    );
}

#[tokio::test]
async fn project_shape_delete_rejects_percent_decoded_traversal() {
    // Joining an unvalidated delete name must make this test fail by removing the sibling file.
    let workspace = TestWorkspace::new();
    let shared = workspace.root.join("shared");
    fs::create_dir_all(shared.join("shapes")).expect("create shapes directory");
    let sibling = shared.join("protected.svg");
    fs::write(&sibling, "protected").expect("seed sibling file");

    let (status, response) = request(
        workspace.app(),
        "DELETE",
        "/project/shapes/%2E%2E%2Fprotected".to_string(),
        Body::empty(),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&response).expect("error response is JSON")["error"]
            ["code"],
        "INVALID_NAME"
    );
    assert!(sibling.is_file(), "unsafe delete must preserve the sibling file");
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
        "-leading",
        "trailing-",
        "two--hyphens",
        "snake_case",
        "naïve",
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
            "PUT",
            format!("/diagrams/{encoded_name}"),
            Body::from(
                serde_json::json!({
                    "version": "1.0",
                    "title": "Unsafe",
                    "theme": "default",
                    "nodes": [],
                    "edges": [],
                    "flows": [],
                    "groups": []
                })
                .to_string(),
            ),
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
            format!("/diagrams/{encoded_name}/render-data"),
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
async fn diagram_routes_reject_percent_decoded_traversal() {
    // Accepting a percent-decoded dot-dot route segment must make this test fail.
    let workspace = TestWorkspace::new();

    for (method, suffix) in [
        ("GET", ""),
        ("POST", "/render"),
        ("GET", "/render-data"),
        ("GET", "/preview"),
        ("DELETE", ""),
    ] {
        assert_invalid_name(
            &workspace,
            method,
            format!("/diagrams/%2e%2e{suffix}"),
            Body::empty(),
        )
        .await;
    }
}

#[tokio::test]
async fn invalid_diagram_name_takes_precedence_over_diagram_validation() {
    // Validating a body before its unsafe route name must make this test fail.
    let workspace = TestWorkspace::new();
    assert_invalid_name(
        &workspace,
        "PUT",
        "/diagrams/..".to_string(),
        Body::from(
            serde_json::json!({
                "version": "1.0",
                "title": "Invalid",
                "theme": "default",
                "nodes": [],
                "edges": [{
                    "id": "missing-nodes",
                    "from": "missing-from",
                    "to": "missing-to"
                }],
                "flows": [],
                "groups": []
            })
            .to_string(),
        ),
    )
    .await;
}

#[tokio::test]
async fn invalid_diagram_name_takes_precedence_over_malformed_json() {
    // Running JSON extraction before route-name validation must make this test fail.
    let workspace = TestWorkspace::new();
    assert_invalid_name(
        &workspace,
        "PUT",
        "/diagrams/..".to_string(),
        Body::from("{"),
    )
    .await;
}

#[tokio::test]
async fn list_diagrams_omits_invalid_on_disk_directory_names() {
    // Returning an invalid on-disk directory as a diagram must make this test fail.
    let workspace = TestWorkspace::new();
    let definition = serde_json::json!({
        "version": "1.0",
        "title": "Listed",
        "theme": "default",
        "nodes": [],
        "edges": [],
        "flows": [],
        "groups": []
    })
    .to_string();
    for name in ["system-overview", "invalid_name"] {
        let directory = workspace.root.join("diagrams").join(name);
        fs::create_dir_all(&directory).expect("create diagram directory");
        fs::write(directory.join("diagram.json"), &definition).expect("write diagram fixture");
    }

    let (status, response) = request(
        workspace.app(),
        "GET",
        "/diagrams".to_string(),
        Body::empty(),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let response: serde_json::Value =
        serde_json::from_str(&response).expect("diagram list response is JSON");
    assert_eq!(
        response["diagrams"]
            .as_array()
            .expect("diagrams is an array")
            .iter()
            .map(|diagram| diagram["name"].as_str().expect("diagram name"))
            .collect::<Vec<_>>(),
        vec!["system-overview"]
    );
}

struct FailingJson;

impl serde::Serialize for FailingJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut sequence = serializer.serialize_seq(Some(2))?;
        sequence.serialize_element("written-before-failure")?;
        Err(serde::ser::Error::custom("injected serialization failure"))
    }
}

#[test]
fn atomic_json_write_removes_temporary_file_after_serialization_failure() {
    // Leaving a temporary sibling after a pre-rename failure must make this test fail.
    let workspace = TestWorkspace::new();
    let shared = workspace.root.join("shared");
    fs::create_dir_all(&shared).expect("create shared directory");
    let destination = shared.join("branding.json");

    assert!(atomic_write_json(&destination, &FailingJson).is_err());
    assert!(!destination.exists());
    assert!(
        fs::read_dir(&shared)
            .expect("read shared directory")
            .next()
            .is_none(),
        "atomic write failure must remove its temporary sibling"
    );
}

#[test]
fn atomic_json_write_removes_temporary_file_after_rename_failure() {
    // Leaving a temporary sibling after rename fails must make this test fail.
    let workspace = TestWorkspace::new();
    let shared = workspace.root.join("shared");
    fs::create_dir_all(&shared).expect("create shared directory");
    let destination = shared.join("branding.json");
    fs::create_dir(&destination).expect("create directory that rename cannot replace with a file");

    assert!(atomic_write_json(&destination, &serde_json::json!({})).is_err());
    assert!(destination.is_dir());
    assert_eq!(
        fs::read_dir(&shared)
            .expect("read shared directory")
            .count(),
        1,
        "rename failure must remove its temporary sibling"
    );
}

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "injected writer failure",
        ))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn serde_json_writer_errors_map_to_io_errors() {
    // Classifying serde_json-wrapped writer failures as serialization errors must fail this test.
    let serde_error = serde_json::to_writer(FailingWriter, &serde_json::json!({ "key": "value" }))
        .expect_err("writer must fail");

    assert!(matches!(
        AtomicJsonWriteError::from(serde_error),
        AtomicJsonWriteError::Io(_)
    ));
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
