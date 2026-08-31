use std::fs;

use axum::{
    Router,
    Json,
    extract::{
        FromRequestParts, Multipart, Path, RawPathParams, State,
        multipart::MultipartRejection,
    },
    http::{StatusCode, request::Parts},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};

use crate::schema::{self, CustomTypes, Diagram, Node, Edge, Flow, Group};
use super::state::{AppState, AtomicJsonWriteError, DiagramPaths, atomic_write_json};

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: String,
    message: String,
}

fn error_response(status: StatusCode, code: &str, message: String) -> (StatusCode, Json<ErrorResponse>) {
    (status, Json(ErrorResponse {
        error: ErrorDetail {
            code: code.to_string(),
            message,
        },
    }))
}

fn diagram_paths(
    state: &AppState,
    name: &str,
) -> Result<DiagramPaths, (StatusCode, Json<ErrorResponse>)> {
    state.diagram_paths(name).map_err(|_| {
        error_response(
            StatusCode::BAD_REQUEST,
            "INVALID_DIAGRAM_NAME",
            format!("Invalid diagram name '{}'", name),
        )
    })
}

struct ValidatedDiagramName(String);

impl FromRequestParts<AppState> for ValidatedDiagramName {
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let params = RawPathParams::from_request_parts(parts, state)
            .await
            .map_err(|_| {
                error_response(
                    StatusCode::BAD_REQUEST,
                    "INVALID_DIAGRAM_NAME",
                    "Invalid diagram name".to_string(),
                )
            })?;
        let name = params
            .iter()
            .find_map(|(key, value)| (key == "name").then_some(value))
            .ok_or_else(|| {
                error_response(
                    StatusCode::BAD_REQUEST,
                    "INVALID_DIAGRAM_NAME",
                    "Invalid diagram name".to_string(),
                )
            })?;
        diagram_paths(state, name)?;
        Ok(Self(name.to_string()))
    }
}

fn write_json<T: Serialize + ?Sized>(
    path: &std::path::Path,
    value: &T,
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    atomic_write_json(path, value).map_err(|error| match error {
        AtomicJsonWriteError::Serialize(error) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERIALIZE_ERROR",
            format!("Failed to serialize: {error}"),
        ),
        AtomicJsonWriteError::Io(error) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "IO_ERROR",
            format!("Failed to write: {error}"),
        ),
    })
}

fn read_diagram(state: &AppState, name: &str) -> Result<Diagram, (StatusCode, Json<ErrorResponse>)> {
    let path = diagram_paths(state, name)?.definition();
    let content = fs::read_to_string(&path).map_err(|_| {
        error_response(StatusCode::NOT_FOUND, "NOT_FOUND", format!("Diagram '{}' not found", name))
    })?;
    serde_json::from_str(&content).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "PARSE_ERROR", format!("Failed to parse diagram: {}", e))
    })
}

fn write_diagram(state: &AppState, name: &str, diagram: &Diagram) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let path = diagram_paths(state, name)?.definition();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", format!("Failed to create directory: {}", e))
        })?;
    }
    write_json(&path, diagram)
}

fn validate(diagram: &Diagram) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    schema::validate_diagram(diagram).map_err(|e| {
        error_response(StatusCode::UNPROCESSABLE_ENTITY, "VALIDATION_FAILED", format!("{}", e))
    })
}

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/project", get(get_project))
        .route("/shared/branding", get(get_shared_branding).put(put_shared_branding))
        .route("/shared/theme", get(get_shared_theme).put(put_shared_theme))
        .route("/types", get(get_resolved_types))
        .route("/project/types", get(get_project_types).put(put_project_types))
        .route("/project/shapes", get(list_shapes).post(upload_shape))
        .route("/project/shapes/{name}", delete(delete_shape))
        .route("/diagrams", get(list_diagrams).post(create_diagram))
        .route("/diagrams/{name}", get(get_diagram).put(put_diagram).delete(delete_diagram))
        .route("/diagrams/{name}/validate", post(validate_diagram))
        .route("/diagrams/{name}/render", post(render_diagram))
        .route("/diagrams/{name}/render-data", get(get_render_data))
        .route("/diagrams/{name}/types", get(get_diagram_resolved_types))
        .route("/diagrams/{name}/preview", get(get_preview))
        .route("/diagrams/{name}/custom-types", get(get_diagram_custom_types).put(put_diagram_custom_types))
        .route("/diagrams/{name}/nodes", get(list_nodes).post(add_node))
        .route("/diagrams/{name}/nodes/{id}", get(get_node).put(update_node).delete(delete_node))
        .route("/diagrams/{name}/edges", get(list_edges).post(add_edge))
        .route("/diagrams/{name}/edges/{id}", get(get_edge).put(update_edge).delete(delete_edge))
        .route("/diagrams/{name}/flows", get(list_flows).post(add_flow))
        .route("/diagrams/{name}/flows/{id}", get(get_flow).put(update_flow).delete(delete_flow))
        .route("/diagrams/{name}/groups", get(list_groups).post(add_group))
        .route("/diagrams/{name}/groups/{id}", get(get_group).put(update_group).delete(delete_group))
}

// --- Project endpoints ---

async fn get_shared_branding(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let path = state.workspace_root.join("shared").join("branding.json");
    let content = fs::read_to_string(&path).map_err(|_| {
        error_response(StatusCode::NOT_FOUND, "NOT_FOUND", "No shared/branding.json found".to_string())
    })?;
    let value = serde_json::from_str(&content).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "PARSE_ERROR", format!("{}", e))
    })?;
    Ok(Json(value))
}

async fn put_shared_branding(
    State(state): State<AppState>,
    Json(value): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    if !value.is_object() {
        return Err(error_response(StatusCode::BAD_REQUEST, "INVALID_BRANDING", "Branding must be a JSON object".to_string()));
    }
    let dir = state.workspace_root.join("shared");
    fs::create_dir_all(&dir).map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", format!("{}", e)))?;
    write_json(&dir.join("branding.json"), &value)?;
    Ok(Json(value))
}

async fn get_shared_theme(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let path = state.workspace_root.join("shared").join("theme.json");
    let content = fs::read_to_string(&path).map_err(|_| {
        error_response(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "No shared/theme.json found".to_string(),
        )
    })?;
    let value = serde_json::from_str(&content).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "PARSE_ERROR",
            format!("{e}"),
        )
    })?;
    Ok(Json(value))
}

async fn put_shared_theme(
    State(state): State<AppState>,
    Json(value): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    if !value.is_object() {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "INVALID_THEME",
            "Theme must be a JSON object".to_string(),
        ));
    }
    let dir = state.workspace_root.join("shared");
    fs::create_dir_all(&dir).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "IO_ERROR",
            format!("{e}"),
        )
    })?;
    write_json(&dir.join("theme.json"), &value)?;
    Ok(Json(value))
}

async fn get_project(State(state): State<AppState>) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let path = state.project_json_path();
    let content = fs::read_to_string(&path).map_err(|_| {
        error_response(StatusCode::NOT_FOUND, "NOT_FOUND", "No project.json found".to_string())
    })?;
    let val: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "PARSE_ERROR", format!("{}", e))
    })?;
    Ok(Json(val))
}

// --- Type endpoints ---

async fn get_resolved_types(
    State(state): State<AppState>,
) -> Result<Json<crate::schema::ResolvedTypeRegistry>, (StatusCode, Json<ErrorResponse>)> {
    // Build a minimal diagram to resolve types at the project level
    let input_path = state.workspace_root.join("project.json");
    let dummy_diagram = Diagram {
        version: "1.0".to_string(),
        title: String::new(),
        description: None,
        theme: "default".to_string(),
        viewport: None,
        nodes: vec![],
        edges: vec![],
        flows: vec![],
        groups: vec![],
        branding: None,
        custom_types: None,
        metadata: Default::default(),
    };
    let registry = crate::workspace::resolve_types(&input_path, &dummy_diagram).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "TYPE_ERROR", format!("{}", e))
    })?;
    Ok(Json(registry))
}

async fn get_diagram_resolved_types(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<crate::schema::ResolvedTypeRegistry>, (StatusCode, Json<ErrorResponse>)> {
    let diagram = read_diagram(&state, &name)?;
    let input_path = diagram_paths(&state, &name)?.definition();
    let registry = crate::workspace::resolve_types(&input_path, &diagram).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "TYPE_ERROR", format!("{}", e))
    })?;
    Ok(Json(registry))
}

async fn get_project_types(
    State(state): State<AppState>,
) -> Result<Json<CustomTypes>, (StatusCode, Json<ErrorResponse>)> {
    let path = state.workspace_root.join("shared").join("types.json");
    let content = fs::read_to_string(&path).map_err(|_| {
        error_response(StatusCode::NOT_FOUND, "NOT_FOUND", "No shared/types.json found".to_string())
    })?;
    let types: CustomTypes = serde_json::from_str(&content).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "PARSE_ERROR", format!("{}", e))
    })?;
    Ok(Json(types))
}

async fn put_project_types(
    State(state): State<AppState>,
    Json(types): Json<CustomTypes>,
) -> Result<Json<CustomTypes>, (StatusCode, Json<ErrorResponse>)> {
    let dir = state.workspace_root.join("shared");
    fs::create_dir_all(&dir).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", format!("{}", e))
    })?;
    let path = dir.join("types.json");
    write_json(&path, &types)?;
    Ok(Json(types))
}

async fn get_diagram_custom_types(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<CustomTypes>, (StatusCode, Json<ErrorResponse>)> {
    let diagram = read_diagram(&state, &name)?;
    let types = diagram.custom_types.unwrap_or(CustomTypes { types: Default::default() });
    Ok(Json(types))
}

async fn put_diagram_custom_types(
    State(state): State<AppState>,
    ValidatedDiagramName(name): ValidatedDiagramName,
    Json(types): Json<CustomTypes>,
) -> Result<Json<CustomTypes>, (StatusCode, Json<ErrorResponse>)> {
    let mut diagram = read_diagram(&state, &name)?;
    diagram.custom_types = Some(types.clone());
    write_diagram(&state, &name, &diagram)?;
    Ok(Json(types))
}

// --- Shape endpoints ---

#[derive(Serialize)]
struct ShapeListEntry {
    name: String,
}

#[derive(Serialize)]
struct ShapeListResponse {
    shapes: Vec<ShapeListEntry>,
}

async fn list_shapes(
    State(state): State<AppState>,
) -> Result<Json<ShapeListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let dir = state.workspace_root.join("shared").join("shapes");
    let mut shapes = Vec::new();
    if dir.is_dir() {
        let entries = fs::read_dir(&dir).map_err(|e| {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", format!("{}", e))
        })?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("svg") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    shapes.push(ShapeListEntry { name: stem.to_string() });
                }
            }
        }
    }
    shapes.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Json(ShapeListResponse { shapes }))
}

async fn upload_shape(
    State(state): State<AppState>,
    multipart: Result<Multipart, MultipartRejection>,
) -> Result<(StatusCode, Json<ShapeListEntry>), (StatusCode, Json<ErrorResponse>)> {
    let mut multipart = multipart.map_err(|e| {
        error_response(
            StatusCode::BAD_REQUEST,
            "INVALID_MULTIPART",
            format!("Invalid multipart shape upload: {e}"),
        )
    })?;
    let mut name = None;
    let mut svg = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error_response(
            StatusCode::BAD_REQUEST,
            "INVALID_MULTIPART",
            format!("Invalid multipart shape upload: {e}"),
        )
    })? {
        match field.name() {
            Some("name") if name.is_none() => {
                name = Some(field.text().await.map_err(|e| {
                    error_response(
                        StatusCode::BAD_REQUEST,
                        "INVALID_MULTIPART",
                        format!("Invalid shape name field: {e}"),
                    )
                })?);
            }
            Some("file") if svg.is_none() => {
                let is_svg_file = field
                    .file_name()
                    .is_some_and(|file_name| file_name.to_ascii_lowercase().ends_with(".svg"));
                let is_svg_media_type = field.content_type() == Some("image/svg+xml");
                if !is_svg_file || !is_svg_media_type {
                    return Err(invalid_svg_response());
                }
                let bytes = field.bytes().await.map_err(|e| {
                    error_response(
                        StatusCode::BAD_REQUEST,
                        "INVALID_MULTIPART",
                        format!("Invalid SVG file field: {e}"),
                    )
                })?;
                svg = Some(String::from_utf8(bytes.to_vec()).map_err(|_| invalid_svg_response())?);
            }
            _ => {
                return Err(error_response(
                    StatusCode::BAD_REQUEST,
                    "INVALID_MULTIPART",
                    "Shape upload requires one name field and one file field".to_string(),
                ));
            }
        }
    }

    let name = name.ok_or_else(|| {
        error_response(
            StatusCode::BAD_REQUEST,
            "INVALID_MULTIPART",
            "Shape upload is missing the name field".to_string(),
        )
    })?;
    let svg = svg.ok_or_else(|| {
        error_response(
            StatusCode::BAD_REQUEST,
            "INVALID_MULTIPART",
            "Shape upload is missing the file field".to_string(),
        )
    })?;

    if !is_safe_shape_name(&name) {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "INVALID_NAME",
            "Invalid shape name".to_string(),
        ));
    }
    if !is_safe_svg(&svg) {
        return Err(invalid_svg_response());
    }

    let dir = state.workspace_root.join("shared").join("shapes");
    fs::create_dir_all(&dir).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", format!("{}", e))
    })?;

    let path = dir.join(format!("{name}.svg"));
    fs::write(&path, &svg).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", format!("{}", e))
    })?;

    Ok((StatusCode::CREATED, Json(ShapeListEntry { name })))
}

fn invalid_svg_response() -> (StatusCode, Json<ErrorResponse>) {
    error_response(
        StatusCode::BAD_REQUEST,
        "INVALID_SVG",
        "File must be a safe SVG with a viewBox".to_string(),
    )
}

fn is_safe_shape_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }

    let mut previous_was_hyphen = false;
    for byte in bytes {
        match byte {
            b'a'..=b'z' | b'0'..=b'9' => previous_was_hyphen = false,
            b'-' if !previous_was_hyphen => previous_was_hyphen = true,
            _ => return false,
        }
    }
    !previous_was_hyphen
}

fn is_safe_svg(svg: &str) -> bool {
    let trimmed = svg.trim();
    let root = if let Some(after_declaration) = trimmed.strip_prefix("<?xml") {
        let Some(end) = after_declaration.find("?>") else {
            return false;
        };
        after_declaration[end + 2..].trim_start()
    } else {
        trimmed
    };
    let Some(root_end) = root.find('>') else {
        return false;
    };
    let opening_tag = &root[..=root_end];
    let opening_tag_lower = opening_tag.to_ascii_lowercase();
    if !opening_tag_lower.starts_with("<svg")
        || !opening_tag_lower
            .as_bytes()
            .get(4)
            .is_some_and(|byte| byte.is_ascii_whitespace() || *byte == b'>')
        || !has_attribute(&opening_tag_lower, "viewbox")
        || !root.trim_end().to_ascii_lowercase().ends_with("</svg>")
    {
        return false;
    }

    let lower = root.to_ascii_lowercase();
    let forbidden = [
        "<script",
        "<foreignobject",
        "<iframe",
        "<object",
        "<embed",
        "<image",
        "<use",
        "<style",
        "<!doctype",
        "<!entity",
        "<?xml-stylesheet",
        "javascript:",
    ];
    !forbidden.iter().any(|value| lower.contains(value)) && !has_event_attribute(&lower)
}

fn has_attribute(opening_tag: &str, attribute: &str) -> bool {
    opening_tag.match_indices(attribute).any(|(index, _)| {
        let before = opening_tag.as_bytes().get(index.wrapping_sub(1));
        let after = opening_tag.as_bytes().get(index + attribute.len());
        before.is_some_and(u8::is_ascii_whitespace)
            && after.is_some_and(|byte| byte.is_ascii_whitespace() || *byte == b'=')
    })
}

fn has_event_attribute(svg: &str) -> bool {
    svg.split('<').skip(1).any(|element| {
        let tag = element.split_once('>').map_or(element, |(tag, _)| tag);
        tag.split_ascii_whitespace().skip(1).any(|attribute| {
            let name = attribute.split_once('=').map_or(attribute, |(name, _)| name);
            name.len() > 2
                && name.starts_with("on")
                && name[2..].bytes().all(|byte| byte.is_ascii_alphabetic())
        })
    })
}

async fn delete_shape(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    if !is_safe_shape_name(&name) {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "INVALID_NAME",
            "Invalid shape name".to_string(),
        ));
    }
    let path = state.workspace_root.join("shared").join("shapes").join(format!("{}.svg", name));
    if !path.exists() {
        return Err(error_response(StatusCode::NOT_FOUND, "NOT_FOUND", format!("Shape '{}' not found", name)));
    }
    fs::remove_file(&path).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", format!("{}", e))
    })?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
struct DiagramListEntry {
    name: String,
    title: String,
    #[serde(rename = "hasOutput")]
    has_output: bool,
}

#[derive(Serialize)]
struct DiagramListResponse {
    diagrams: Vec<DiagramListEntry>,
}

async fn list_diagrams(State(state): State<AppState>) -> Result<Json<DiagramListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let diagrams_dir = state.diagrams_dir();
    let mut entries = Vec::new();

    if diagrams_dir.exists() {
        let read_dir = fs::read_dir(&diagrams_dir).map_err(|e| {
            error_response(StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", format!("{}", e))
        })?;

        for entry in read_dir.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let Ok(paths) = state.diagram_paths(&name) else {
                continue;
            };
            let diagram_path = paths.definition();
            if !diagram_path.exists() {
                continue;
            }

            let title = fs::read_to_string(&diagram_path)
                .ok()
                .and_then(|s| serde_json::from_str::<Diagram>(&s).ok())
                .map(|d| d.title)
                .unwrap_or_else(|| name.clone());

            let has_output = paths.output_directory().exists();

            entries.push(DiagramListEntry { name, title, has_output });
        }
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Json(DiagramListResponse { diagrams: entries }))
}

// --- Diagram CRUD ---

async fn get_diagram(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Diagram>, (StatusCode, Json<ErrorResponse>)> {
    let diagram = read_diagram(&state, &name)?;
    Ok(Json(diagram))
}

async fn put_diagram(
    State(state): State<AppState>,
    ValidatedDiagramName(name): ValidatedDiagramName,
    Json(diagram): Json<Diagram>,
) -> Result<Json<Diagram>, (StatusCode, Json<ErrorResponse>)> {
    validate(&diagram)?;
    write_diagram(&state, &name, &diagram)?;
    Ok(Json(diagram))
}

#[derive(Deserialize)]
struct CreateDiagramRequest {
    name: String,
    title: String,
}

async fn create_diagram(
    State(state): State<AppState>,
    Json(req): Json<CreateDiagramRequest>,
) -> Result<(StatusCode, Json<Diagram>), (StatusCode, Json<ErrorResponse>)> {
    let path = diagram_paths(&state, &req.name)?.definition();
    if path.exists() {
        return Err(error_response(StatusCode::CONFLICT, "ALREADY_EXISTS", format!("Diagram '{}' already exists", req.name)));
    }

    let diagram = Diagram {
        version: "1.0".to_string(),
        title: req.title,
        description: None,
        theme: "default".to_string(),
        viewport: None,
        nodes: vec![],
        edges: vec![],
        flows: vec![],
        groups: vec![],
        branding: None,
        custom_types: None,
        metadata: Default::default(),
    };

    write_diagram(&state, &req.name, &diagram)?;
    Ok((StatusCode::CREATED, Json(diagram)))
}

async fn delete_diagram(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let paths = diagram_paths(&state, &name)?;
    let dir = paths.directory();
    if !dir.exists() {
        return Err(error_response(StatusCode::NOT_FOUND, "NOT_FOUND", format!("Diagram '{}' not found", name)));
    }
    fs::remove_dir_all(dir).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", format!("{}", e))
    })?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Action endpoints ---

#[derive(Serialize)]
struct ValidateResponse {
    valid: bool,
}

async fn validate_diagram(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ValidateResponse>, (StatusCode, Json<ErrorResponse>)> {
    let diagram = read_diagram(&state, &name)?;
    validate(&diagram)?;
    Ok(Json(ValidateResponse { valid: true }))
}

#[derive(Serialize)]
struct RenderResponse {
    #[serde(rename = "outputPath")]
    output_path: String,
}

async fn render_diagram(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<RenderResponse>, (StatusCode, Json<ErrorResponse>)> {
    let diagram = read_diagram(&state, &name)?;
    validate(&diagram)?;

    let paths = diagram_paths(&state, &name)?;
    let input_path = paths.definition();
    let (html, _) = crate::workspace::render_pipeline(&input_path, &diagram).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "RENDER_ERROR", format!("{}", e))
    })?;

    let output_dir = paths.output_directory();
    fs::create_dir_all(&output_dir).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", format!("{}", e))
    })?;

    let output_path = paths.output();
    fs::write(&output_path, html).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", format!("{}", e))
    })?;

    let relative = format!("diagrams/{0}/output/{0}.html", paths.name());
    Ok(Json(RenderResponse { output_path: relative }))
}

async fn get_render_data(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<crate::render::DiagramRenderData>, (StatusCode, Json<ErrorResponse>)> {
    let diagram = read_diagram(&state, &name)?;
    validate(&diagram)?;

    let input_path = diagram_paths(&state, &name)?.definition();
    let branding = crate::workspace::resolve_branding(&input_path, &diagram).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "BRANDING_ERROR", format!("{}", e))
    })?;

    let type_registry = crate::workspace::resolve_types(&input_path, &diagram).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "TYPE_ERROR", format!("{}", e))
    })?;

    let shape_overrides = crate::workspace::load_shape_overrides(&input_path, &type_registry).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "SHAPE_ERROR", format!("{}", e))
    })?;

    let layout_data = crate::layout::compute_layout(&diagram, &type_registry).map_err(|e| {
        error_response(StatusCode::INTERNAL_SERVER_ERROR, "LAYOUT_ERROR", format!("{}", e))
    })?;

    let render_data = crate::render::build_render_data(
        &diagram, &layout_data, branding.as_ref(),
        Some(&type_registry), Some(&shape_overrides),
    );
    Ok(Json(render_data))
}

async fn get_preview(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<axum::response::Html<String>, (StatusCode, Json<ErrorResponse>)> {
    let output_path = diagram_paths(&state, &name)?.output();
    let content = fs::read_to_string(&output_path).map_err(|_| {
        error_response(StatusCode::NOT_FOUND, "NOT_FOUND", "No rendered output found. Call POST /render first.".to_string())
    })?;
    Ok(axum::response::Html(content))
}

// --- Element CRUD: Nodes ---

async fn list_nodes(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Vec<Node>>, (StatusCode, Json<ErrorResponse>)> {
    let diagram = read_diagram(&state, &name)?;
    Ok(Json(diagram.nodes))
}

async fn get_node(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<Json<Node>, (StatusCode, Json<ErrorResponse>)> {
    let diagram = read_diagram(&state, &name)?;
    let node = diagram.nodes.iter().find(|n| n.id == id).ok_or_else(|| {
        error_response(StatusCode::NOT_FOUND, "NOT_FOUND", format!("Node '{}' not found", id))
    })?;
    Ok(Json(node.clone()))
}

async fn add_node(
    State(state): State<AppState>,
    ValidatedDiagramName(name): ValidatedDiagramName,
    Json(node): Json<Node>,
) -> Result<(StatusCode, Json<Node>), (StatusCode, Json<ErrorResponse>)> {
    let mut diagram = read_diagram(&state, &name)?;
    if diagram.nodes.iter().any(|n| n.id == node.id) {
        return Err(error_response(StatusCode::CONFLICT, "ALREADY_EXISTS", format!("Node '{}' already exists", node.id)));
    }
    diagram.nodes.push(node.clone());
    validate(&diagram)?;
    write_diagram(&state, &name, &diagram)?;
    Ok((StatusCode::CREATED, Json(node)))
}

async fn update_node(
    State(state): State<AppState>,
    ValidatedDiagramName(name): ValidatedDiagramName,
    Path((_, id)): Path<(String, String)>,
    Json(node): Json<Node>,
) -> Result<Json<Node>, (StatusCode, Json<ErrorResponse>)> {
    let mut diagram = read_diagram(&state, &name)?;
    let idx = diagram.nodes.iter().position(|n| n.id == id).ok_or_else(|| {
        error_response(StatusCode::NOT_FOUND, "NOT_FOUND", format!("Node '{}' not found", id))
    })?;
    diagram.nodes[idx] = node.clone();
    validate(&diagram)?;
    write_diagram(&state, &name, &diagram)?;
    Ok(Json(node))
}

async fn delete_node(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut diagram = read_diagram(&state, &name)?;
    let len_before = diagram.nodes.len();
    diagram.nodes.retain(|n| n.id != id);
    if diagram.nodes.len() == len_before {
        return Err(error_response(StatusCode::NOT_FOUND, "NOT_FOUND", format!("Node '{}' not found", id)));
    }

    // Cascade: remove edges referencing this node
    let removed_edge_ids: Vec<String> = diagram.edges.iter()
        .filter(|e| e.from == id || e.to == id)
        .map(|e| e.id.clone())
        .collect();
    diagram.edges.retain(|e| e.from != id && e.to != id);

    // Cascade: remove flow steps referencing removed edges
    for flow in &mut diagram.flows {
        flow.steps.retain(|s| !removed_edge_ids.contains(&s.edge));
    }

    // Remove from groups
    for group in &mut diagram.groups {
        group.nodes.retain(|n| n != &id);
    }

    write_diagram(&state, &name, &diagram)?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Element CRUD: Edges ---

async fn list_edges(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Vec<Edge>>, (StatusCode, Json<ErrorResponse>)> {
    let diagram = read_diagram(&state, &name)?;
    Ok(Json(diagram.edges))
}

async fn get_edge(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<Json<Edge>, (StatusCode, Json<ErrorResponse>)> {
    let diagram = read_diagram(&state, &name)?;
    let edge = diagram.edges.iter().find(|e| e.id == id).ok_or_else(|| {
        error_response(StatusCode::NOT_FOUND, "NOT_FOUND", format!("Edge '{}' not found", id))
    })?;
    Ok(Json(edge.clone()))
}

async fn add_edge(
    State(state): State<AppState>,
    ValidatedDiagramName(name): ValidatedDiagramName,
    Json(edge): Json<Edge>,
) -> Result<(StatusCode, Json<Edge>), (StatusCode, Json<ErrorResponse>)> {
    let mut diagram = read_diagram(&state, &name)?;
    if diagram.edges.iter().any(|e| e.id == edge.id) {
        return Err(error_response(StatusCode::CONFLICT, "ALREADY_EXISTS", format!("Edge '{}' already exists", edge.id)));
    }
    diagram.edges.push(edge.clone());
    validate(&diagram)?;
    write_diagram(&state, &name, &diagram)?;
    Ok((StatusCode::CREATED, Json(edge)))
}

async fn update_edge(
    State(state): State<AppState>,
    ValidatedDiagramName(name): ValidatedDiagramName,
    Path((_, id)): Path<(String, String)>,
    Json(edge): Json<Edge>,
) -> Result<Json<Edge>, (StatusCode, Json<ErrorResponse>)> {
    let mut diagram = read_diagram(&state, &name)?;
    let idx = diagram.edges.iter().position(|e| e.id == id).ok_or_else(|| {
        error_response(StatusCode::NOT_FOUND, "NOT_FOUND", format!("Edge '{}' not found", id))
    })?;
    diagram.edges[idx] = edge.clone();
    validate(&diagram)?;
    write_diagram(&state, &name, &diagram)?;
    Ok(Json(edge))
}

async fn delete_edge(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut diagram = read_diagram(&state, &name)?;
    let len_before = diagram.edges.len();
    diagram.edges.retain(|e| e.id != id);
    if diagram.edges.len() == len_before {
        return Err(error_response(StatusCode::NOT_FOUND, "NOT_FOUND", format!("Edge '{}' not found", id)));
    }

    // Cascade: remove flow steps referencing this edge
    for flow in &mut diagram.flows {
        flow.steps.retain(|s| s.edge != id);
    }

    write_diagram(&state, &name, &diagram)?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Element CRUD: Flows ---

async fn list_flows(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Vec<Flow>>, (StatusCode, Json<ErrorResponse>)> {
    let diagram = read_diagram(&state, &name)?;
    Ok(Json(diagram.flows))
}

async fn get_flow(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<Json<Flow>, (StatusCode, Json<ErrorResponse>)> {
    let diagram = read_diagram(&state, &name)?;
    let flow = diagram.flows.iter().find(|f| f.id == id).ok_or_else(|| {
        error_response(StatusCode::NOT_FOUND, "NOT_FOUND", format!("Flow '{}' not found", id))
    })?;
    Ok(Json(flow.clone()))
}

async fn add_flow(
    State(state): State<AppState>,
    ValidatedDiagramName(name): ValidatedDiagramName,
    Json(flow): Json<Flow>,
) -> Result<(StatusCode, Json<Flow>), (StatusCode, Json<ErrorResponse>)> {
    let mut diagram = read_diagram(&state, &name)?;
    if diagram.flows.iter().any(|f| f.id == flow.id) {
        return Err(error_response(StatusCode::CONFLICT, "ALREADY_EXISTS", format!("Flow '{}' already exists", flow.id)));
    }
    diagram.flows.push(flow.clone());
    validate(&diagram)?;
    write_diagram(&state, &name, &diagram)?;
    Ok((StatusCode::CREATED, Json(flow)))
}

async fn update_flow(
    State(state): State<AppState>,
    ValidatedDiagramName(name): ValidatedDiagramName,
    Path((_, id)): Path<(String, String)>,
    Json(flow): Json<Flow>,
) -> Result<Json<Flow>, (StatusCode, Json<ErrorResponse>)> {
    let mut diagram = read_diagram(&state, &name)?;
    let idx = diagram.flows.iter().position(|f| f.id == id).ok_or_else(|| {
        error_response(StatusCode::NOT_FOUND, "NOT_FOUND", format!("Flow '{}' not found", id))
    })?;
    diagram.flows[idx] = flow.clone();
    validate(&diagram)?;
    write_diagram(&state, &name, &diagram)?;
    Ok(Json(flow))
}

async fn delete_flow(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut diagram = read_diagram(&state, &name)?;
    let len_before = diagram.flows.len();
    diagram.flows.retain(|f| f.id != id);
    if diagram.flows.len() == len_before {
        return Err(error_response(StatusCode::NOT_FOUND, "NOT_FOUND", format!("Flow '{}' not found", id)));
    }
    write_diagram(&state, &name, &diagram)?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Element CRUD: Groups ---

async fn list_groups(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Vec<Group>>, (StatusCode, Json<ErrorResponse>)> {
    let diagram = read_diagram(&state, &name)?;
    Ok(Json(diagram.groups))
}

async fn get_group(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<Json<Group>, (StatusCode, Json<ErrorResponse>)> {
    let diagram = read_diagram(&state, &name)?;
    let group = diagram.groups.iter().find(|g| g.id == id).ok_or_else(|| {
        error_response(StatusCode::NOT_FOUND, "NOT_FOUND", format!("Group '{}' not found", id))
    })?;
    Ok(Json(group.clone()))
}

async fn add_group(
    State(state): State<AppState>,
    ValidatedDiagramName(name): ValidatedDiagramName,
    Json(group): Json<Group>,
) -> Result<(StatusCode, Json<Group>), (StatusCode, Json<ErrorResponse>)> {
    let mut diagram = read_diagram(&state, &name)?;
    if diagram.groups.iter().any(|g| g.id == group.id) {
        return Err(error_response(StatusCode::CONFLICT, "ALREADY_EXISTS", format!("Group '{}' already exists", group.id)));
    }
    diagram.groups.push(group.clone());
    validate(&diagram)?;
    write_diagram(&state, &name, &diagram)?;
    Ok((StatusCode::CREATED, Json(group)))
}

async fn update_group(
    State(state): State<AppState>,
    ValidatedDiagramName(name): ValidatedDiagramName,
    Path((_, id)): Path<(String, String)>,
    Json(group): Json<Group>,
) -> Result<Json<Group>, (StatusCode, Json<ErrorResponse>)> {
    let mut diagram = read_diagram(&state, &name)?;
    let idx = diagram.groups.iter().position(|g| g.id == id).ok_or_else(|| {
        error_response(StatusCode::NOT_FOUND, "NOT_FOUND", format!("Group '{}' not found", id))
    })?;
    diagram.groups[idx] = group.clone();
    validate(&diagram)?;
    write_diagram(&state, &name, &diagram)?;
    Ok(Json(group))
}

async fn delete_group(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut diagram = read_diagram(&state, &name)?;
    let len_before = diagram.groups.len();
    diagram.groups.retain(|g| g.id != id);
    if diagram.groups.len() == len_before {
        return Err(error_response(StatusCode::NOT_FOUND, "NOT_FOUND", format!("Group '{}' not found", id)));
    }

    // Remove references to this group from other groups' sub-groups
    for group in &mut diagram.groups {
        group.groups.retain(|g| g != &id);
    }

    write_diagram(&state, &name, &diagram)?;
    Ok(StatusCode::NO_CONTENT)
}
