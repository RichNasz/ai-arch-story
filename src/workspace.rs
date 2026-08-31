use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::schema::{CustomTypes, DiagramBranding, ResolvedTypeRegistry};
use crate::render::ResolvedBranding;

pub fn resolve_branding(
    input_path: &Path,
    diagram: &crate::schema::Diagram,
) -> Result<Option<ResolvedBranding>> {
    if let Some(ref db) = diagram.branding {
        if db.enabled == Some(false) {
            return Ok(None);
        }
    }

    let project_root = find_project_root(input_path);
    let shared_dir = project_root.as_ref().map(|r| r.join("shared"));

    let project_branding: Option<DiagramBranding> = shared_dir
        .as_ref()
        .map(|d| d.join("branding.json"))
        .filter(|p| p.exists())
        .map(|p| {
            let s = fs::read_to_string(&p)
                .with_context(|| format!("Failed to read {}", p.display()))?;
            serde_json::from_str(&s)
                .with_context(|| format!("Failed to parse {}", p.display()))
        })
        .transpose()?;

    let merged = merge_branding(project_branding, diagram.branding.clone());

    let branding = match merged {
        Some(b) => b,
        None => return Ok(None),
    };

    let base_dir = shared_dir
        .as_deref()
        .or_else(|| input_path.parent())
        .unwrap_or_else(|| Path::new("."));

    let logo_data_uri = branding
        .logo
        .as_ref()
        .map(|logo| resolve_asset_to_data_uri(base_dir, &logo.src))
        .transpose()?;

    let favicon_data_uri = branding
        .favicon
        .as_ref()
        .map(|fav| resolve_asset_to_data_uri(base_dir, &fav.src))
        .transpose()?;

    let logo_alt = branding
        .logo
        .as_ref()
        .and_then(|l| l.alt.clone())
        .or_else(|| branding.organization.clone());

    Ok(Some(ResolvedBranding {
        organization: branding.organization,
        logo_data_uri,
        logo_alt,
        logo_placement: branding
            .logo
            .as_ref()
            .and_then(|l| l.placement.clone())
            .unwrap_or_else(|| "header".to_string()),
        logo_height: branding.logo.as_ref().and_then(|l| l.height).unwrap_or(24),
        primary_color: branding.colors.as_ref().and_then(|c| c.primary.clone()),
        secondary_color: branding.colors.as_ref().and_then(|c| c.secondary.clone()),
        footer_text: branding.footer.as_ref().and_then(|f| f.text.clone()),
        show_generated_date: branding
            .footer
            .as_ref()
            .and_then(|f| f.show_generated_date)
            .unwrap_or(false),
        favicon_data_uri,
    }))
}

pub fn find_project_root(input_path: &Path) -> Option<PathBuf> {
    let mut dir = input_path.parent()?;
    for _ in 0..5 {
        if dir.join("project.json").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
    None
}

fn merge_branding(
    project: Option<DiagramBranding>,
    diagram: Option<DiagramBranding>,
) -> Option<DiagramBranding> {
    match (project, diagram) {
        (None, None) => None,
        (Some(p), None) => Some(p),
        (None, Some(d)) => Some(d),
        (Some(p), Some(d)) => Some(DiagramBranding {
            enabled: d.enabled.or(p.enabled),
            organization: d.organization.or(p.organization),
            logo: d.logo.or(p.logo),
            colors: match (p.colors, d.colors) {
                (None, None) => None,
                (Some(c), None) | (None, Some(c)) => Some(c),
                (Some(pc), Some(dc)) => Some(crate::schema::BrandingColors {
                    primary: dc.primary.or(pc.primary),
                    secondary: dc.secondary.or(pc.secondary),
                }),
            },
            footer: d.footer.or(p.footer),
            favicon: d.favicon.or(p.favicon),
        }),
    }
}

pub fn resolve_asset_to_data_uri(base_dir: &Path, src: &str) -> Result<String> {
    if src.starts_with("data:") {
        return Ok(src.to_string());
    }

    let path = base_dir.join(src);
    let bytes = fs::read(&path)
        .with_context(|| format!("Failed to read branding asset {}", path.display()))?;

    let mime = if src.ends_with(".svg") {
        "image/svg+xml"
    } else if src.ends_with(".png") {
        "image/png"
    } else if src.ends_with(".jpg") || src.ends_with(".jpeg") {
        "image/jpeg"
    } else {
        "application/octet-stream"
    };

    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let encoded = STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, encoded))
}

#[derive(Deserialize, Default)]
struct ProjectJson {
    #[serde(default)]
    type_libraries: Vec<TypeLibraryRef>,
}

#[derive(Deserialize)]
struct TypeLibraryRef {
    path: String,
}

pub fn resolve_types(
    input_path: &Path,
    diagram: &crate::schema::Diagram,
) -> Result<ResolvedTypeRegistry> {
    let mut registry = ResolvedTypeRegistry::built_in();

    let project_root = find_project_root(input_path);

    // Load type libraries from project.json
    if let Some(ref root) = project_root {
        let project_json_path = root.join("project.json");
        if project_json_path.exists() {
            let content = fs::read_to_string(&project_json_path)
                .with_context(|| format!("Failed to read {}", project_json_path.display()))?;
            let project: ProjectJson = serde_json::from_str(&content).unwrap_or_default();

            for lib_ref in &project.type_libraries {
                let lib_dir = root.join(&lib_ref.path);
                let types_path = lib_dir.join("types.json");
                if types_path.exists() {
                    let s = fs::read_to_string(&types_path)
                        .with_context(|| format!("Failed to read type library {}", types_path.display()))?;
                    let custom: CustomTypes = serde_json::from_str(&s)
                        .with_context(|| format!("Failed to parse type library {}", types_path.display()))?;
                    registry.merge(&custom);
                }
            }
        }
    }

    // Load project-level types
    if let Some(ref root) = project_root {
        let shared_types = root.join("shared").join("types.json");
        if shared_types.exists() {
            let s = fs::read_to_string(&shared_types)
                .with_context(|| format!("Failed to read {}", shared_types.display()))?;
            let custom: CustomTypes = serde_json::from_str(&s)
                .with_context(|| format!("Failed to parse {}", shared_types.display()))?;
            registry.merge(&custom);
        }
    }

    // Load diagram-level custom types
    if let Some(ref ct) = diagram.custom_types {
        registry.merge(ct);
    }

    Ok(registry)
}

pub fn load_shape_overrides(
    input_path: &Path,
    registry: &ResolvedTypeRegistry,
) -> Result<HashMap<String, String>> {
    let mut overrides: HashMap<String, String> = HashMap::new();
    let project_root = find_project_root(input_path);

    // Collect all shape names used by the registry
    let shape_names: std::collections::HashSet<&str> = registry.types.values().map(|t| t.shape.as_str()).collect();

    // Search for SVG overrides in project shared/shapes/
    if let Some(ref root) = project_root {
        let shapes_dir = root.join("shared").join("shapes");
        if shapes_dir.is_dir() {
            load_svg_dir(&shapes_dir, &shape_names, &mut overrides)?;
        }
    }

    // Search for SVG overrides in diagram assets/shapes/
    if let Some(parent) = input_path.parent() {
        let diagram_shapes = parent.join("assets").join("shapes");
        if diagram_shapes.is_dir() {
            load_svg_dir(&diagram_shapes, &shape_names, &mut overrides)?;
        }
    }

    Ok(overrides)
}

fn load_svg_dir(
    dir: &Path,
    _shape_names: &std::collections::HashSet<&str>,
    overrides: &mut HashMap<String, String>,
) -> Result<()> {
    for entry in fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("svg") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let svg_content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read SVG {}", path.display()))?;
                overrides.insert(stem.to_string(), svg_content);
            }
        }
    }
    Ok(())
}

use std::collections::HashMap as StdHashMap;
type HashMap<K, V> = StdHashMap<K, V>;

pub fn render_pipeline(
    input_path: &Path,
    diagram: &crate::schema::Diagram,
) -> Result<(String, Option<String>)> {
    let branding = resolve_branding(input_path, diagram)?;
    let type_registry = resolve_types(input_path, diagram)?;
    let shape_overrides = load_shape_overrides(input_path, &type_registry)?;

    let layout_data = crate::layout::compute_layout(diagram, &type_registry)
        .with_context(|| "Layout computation failed")?;

    let render_data = crate::render::build_render_data(
        diagram, &layout_data, branding.as_ref(),
        Some(&type_registry), Some(&shape_overrides),
    );

    let favicon_uri = branding.as_ref().and_then(|b| b.favicon_data_uri.as_deref());
    let html = crate::export::assemble_html_with_favicon(&render_data, favicon_uri)
        .with_context(|| "Failed to assemble HTML")?;

    let favicon_owned = branding.and_then(|b| b.favicon_data_uri);
    Ok((html, favicon_owned))
}
