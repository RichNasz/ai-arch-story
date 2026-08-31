use serde::Serialize;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::schema::{Diagram, EdgeDirection, ResolvedTypeRegistry};

#[derive(Debug, Clone, Serialize)]
pub struct LayoutData {
    pub width: f64,
    pub height: f64,
    pub nodes: Vec<LayoutNode>,
    pub edges: Vec<LayoutEdge>,
    pub groups: Vec<LayoutGroup>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LayoutNode {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub shape: String,
    pub accent_color: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LayoutEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub direction: String,
    pub path: String,
    pub label_position: Option<LabelPosition>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LabelPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LayoutGroup {
    pub id: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub depth: u32,
    pub style: Option<GroupLayoutStyle>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupLayoutStyle {
    pub color: Option<String>,
    pub border: Option<String>,
}

pub fn compute_layout(diagram: &Diagram, type_registry: &ResolvedTypeRegistry) -> anyhow::Result<LayoutData> {
    let viewport = diagram.viewport.unwrap_or_default();
    let vw = viewport.width;
    let vh = viewport.height;

    // Generate dot source
    let dot = generate_dot(diagram, vw, vh, type_registry);

    // Run dot -Tjson0
    let gv_json = run_graphviz(&dot)?;

    // Parse Graphviz JSON output
    let gv: serde_json::Value = serde_json::from_str(&gv_json)
        .map_err(|e| anyhow::anyhow!("Failed to parse Graphviz JSON: {}", e))?;

    // Extract bounding box — Graphviz uses bottom-left origin, we need top-left
    let bb = parse_bb(gv.get("bb").and_then(|v| v.as_str()).unwrap_or("0,0,1920,1080"));
    let gv_width = bb.2 - bb.0;
    let gv_height = bb.3 - bb.1;

    // Scale uniformly to fill viewport with 5% padding on each side
    let padding = 0.05;
    let usable_w = vw * (1.0 - 2.0 * padding);
    let usable_h = vh * (1.0 - 2.0 * padding);
    let scale_x = usable_w / gv_width.max(1.0);
    let scale_y = usable_h / gv_height.max(1.0);
    let scale = scale_x.min(scale_y);
    let offset_x = (vw - gv_width * scale) / 2.0;
    let offset_y = (vh - gv_height * scale) / 2.0;

    // Build node lookup from Graphviz output
    let gv_objects = gv.get("objects").and_then(|v| v.as_array());

    // Parse nodes
    let mut node_map: HashMap<String, LayoutNode> = HashMap::new();

    if let Some(objects) = gv_objects {
        for obj in objects {
            // Graphviz nests cluster subgraphs as objects too — skip those
            if obj.get("subgraphs").is_some() || obj.get("nodes").is_some() {
                continue;
            }

            let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if name.is_empty() {
                continue;
            }

            let pos = parse_point(obj.get("pos").and_then(|v| v.as_str()).unwrap_or("0,0"));
            let w_inches: f64 = obj
                .get("width")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.5);
            let h_inches: f64 = obj
                .get("height")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.75);

            // Graphviz sizes are in inches, positions in points (72 dpi)
            let node_w = w_inches * 72.0 * scale;
            let node_h = h_inches * 72.0 * scale;

            // Graphviz pos is center of node, Y is bottom-up
            let cx = pos.0 * scale + offset_x;
            let cy = (gv_height - pos.1) * scale + offset_y;
            let x = cx - node_w / 2.0;
            let y = cy - node_h / 2.0;

            // Find matching diagram node
            if let Some(dnode) = diagram.nodes.iter().find(|n| n.id == name) {
                let type_key = dnode.node_type.as_ref().map_or("generic".to_string(), |nt| nt.as_key());
                let resolved = type_registry.resolve(&type_key);
                node_map.insert(
                    name.to_string(),
                    LayoutNode {
                        id: name.to_string(),
                        label: dnode.label.clone(),
                        node_type: type_key,
                        shape: resolved.shape.clone(),
                        accent_color: resolved.accent_color.clone(),
                        x,
                        y,
                        width: node_w,
                        height: node_h,
                        metadata: dnode.metadata.clone(),
                    },
                );
            }
        }
    }

    // Parse edges
    let mut layout_edges: Vec<LayoutEdge> = Vec::new();

    if let Some(gv_edges) = gv.get("edges").and_then(|v| v.as_array()) {
        for gv_edge in gv_edges {
            let tail = gv_edge.get("tail").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let head = gv_edge.get("head").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

            // Resolve tail/head node IDs from gv_objects
            let from_id = gv_objects
                .and_then(|objs| objs.get(tail))
                .and_then(|o| o.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let to_id = gv_objects
                .and_then(|objs| objs.get(head))
                .and_then(|o| o.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Find matching diagram edge
            let dedge = diagram
                .edges
                .iter()
                .find(|e| e.from == from_id && e.to == to_id);

            let edge_id = dedge
                .map(|e| e.id.clone())
                .unwrap_or_else(|| format!("e-{}-{}", from_id, to_id));

            // Parse spline points from Graphviz
            let path = if let Some(pos_str) = gv_edge.get("pos").and_then(|v| v.as_str()) {
                convert_gv_spline(pos_str, scale, gv_height, offset_x, offset_y)
            } else {
                // Fallback: straight line
                let from_node = node_map.get(&from_id);
                let to_node = node_map.get(&to_id);
                match (from_node, to_node) {
                    (Some(f), Some(t)) => format!(
                        "M{:.1},{:.1} L{:.1},{:.1}",
                        f.x + f.width / 2.0,
                        f.y + f.height / 2.0,
                        t.x + t.width / 2.0,
                        t.y + t.height / 2.0
                    ),
                    _ => String::new(),
                }
            };

            let direction = dedge
                .and_then(|e| e.direction)
                .unwrap_or_default();

            let label = dedge.and_then(|e| e.label.clone());

            // Label position at spline midpoint
            let label_position = if let Some(lp) = gv_edge.get("lp").and_then(|v| v.as_str()) {
                let pt = parse_point(lp);
                Some(LabelPosition {
                    x: pt.0 * scale + offset_x,
                    y: (gv_height - pt.1) * scale + offset_y,
                })
            } else {
                None
            };

            layout_edges.push(LayoutEdge {
                id: edge_id,
                from: from_id,
                to: to_id,
                label,
                direction: match direction {
                    EdgeDirection::OneWay => "one-way".to_string(),
                    EdgeDirection::TwoWay => "two-way".to_string(),
                    EdgeDirection::None => "none".to_string(),
                },
                path,
                label_position,
            });
        }
    }

    // Parse groups from cluster subgraphs
    let group_depths = compute_group_depths(diagram);
    let mut layout_groups: Vec<LayoutGroup> = Vec::new();

    if let Some(objects) = gv_objects {
        for obj in objects {
            let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if !name.starts_with("cluster_") {
                continue;
            }
            let group_id = &name["cluster_".len()..];

            let dgroup = diagram.groups.iter().find(|g| g.id == group_id);
            if dgroup.is_none() {
                continue;
            }
            let dgroup = dgroup.unwrap();

            let cbb = parse_bb(obj.get("bb").and_then(|v| v.as_str()).unwrap_or("0,0,100,100"));
            let gx = cbb.0 * scale + offset_x;
            let gy = (gv_height - cbb.3) * scale + offset_y;
            let gw = (cbb.2 - cbb.0) * scale;
            let gh = (cbb.3 - cbb.1) * scale;

            let depth = group_depths.get(group_id).copied().unwrap_or(0);
            let style = dgroup.style.as_ref().map(|s| GroupLayoutStyle {
                color: s.color.clone(),
                border: s.border.clone(),
            });

            layout_groups.push(LayoutGroup {
                id: group_id.to_string(),
                label: dgroup.label.clone(),
                x: gx,
                y: gy,
                width: gw,
                height: gh,
                depth,
                style,
            });
        }
    }

    let layout_nodes: Vec<LayoutNode> = diagram
        .nodes
        .iter()
        .filter_map(|n| node_map.remove(&n.id))
        .collect();

    Ok(LayoutData {
        width: vw,
        height: vh,
        nodes: layout_nodes,
        edges: layout_edges,
        groups: layout_groups,
    })
}

fn generate_dot(diagram: &Diagram, _vw: f64, _vh: f64, type_registry: &ResolvedTypeRegistry) -> String {
    let mut dot = String::new();
    dot.push_str("digraph G {\n");
    dot.push_str("  rankdir=LR;\n");
    dot.push_str("  pad=1.0;\n");
    dot.push_str("  nodesep=1.0;\n");
    dot.push_str("  ranksep=2.0;\n");
    dot.push_str("  splines=true;\n");
    dot.push_str("  compound=true;\n");
    dot.push_str("  node [shape=box, style=rounded, fontsize=20, fixedsize=true];\n");
    dot.push_str("  edge [fontsize=14];\n");

    // Build set of grouped node IDs
    let mut grouped_nodes: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for group in &diagram.groups {
        for nid in &group.nodes {
            grouped_nodes.insert(nid.as_str());
        }
    }

    // Emit groups as cluster subgraphs
    for group in &diagram.groups {
        emit_cluster(&mut dot, group, diagram, "  ", type_registry);
    }

    // Emit ungrouped nodes
    for node in &diagram.nodes {
        if !grouped_nodes.contains(node.id.as_str()) {
            let type_key = node.node_type.as_ref().map_or("generic", |nt| match nt {
                crate::schema::NodeType::BuiltIn(b) => b.as_key(),
                crate::schema::NodeType::Custom(s) => s.as_str(),
            });
            let shape = &type_registry.resolve(type_key).gv_shape;
            let (w, h) = node_dimensions(&node.label);
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\", shape={}, width={:.1}, height={:.1}];\n",
                node.id,
                dot_escape(&node.label),
                shape,
                w,
                h
            ));
        }
    }

    // Emit edges
    for edge in &diagram.edges {
        let attrs = match &edge.label {
            Some(lbl) => format!(" [label=\"{}\"]", dot_escape(lbl)),
            None => String::new(),
        };
        dot.push_str(&format!("  \"{}\" -> \"{}\"{};\n", edge.from, edge.to, attrs));
    }

    dot.push_str("}\n");
    dot
}

fn emit_cluster(dot: &mut String, group: &crate::schema::Group, diagram: &Diagram, indent: &str, type_registry: &ResolvedTypeRegistry) {
    dot.push_str(&format!("{}subgraph \"cluster_{}\" {{\n", indent, group.id));
    dot.push_str(&format!("{}  label=\"{}\";\n", indent, dot_escape(&group.label)));
    dot.push_str(&format!("{}  style=rounded;\n", indent));
    dot.push_str(&format!("{}  margin=16;\n", indent));

    // Emit nested sub-groups
    for sub_id in &group.groups {
        if let Some(sub) = diagram.groups.iter().find(|g| g.id == *sub_id) {
            emit_cluster(dot, sub, diagram, &format!("{}  ", indent), type_registry);
        }
    }

    // Emit nodes in this group
    for nid in &group.nodes {
        if let Some(node) = diagram.nodes.iter().find(|n| n.id == *nid) {
            let type_key = node.node_type.as_ref().map_or("generic", |nt| match nt {
                crate::schema::NodeType::BuiltIn(b) => b.as_key(),
                crate::schema::NodeType::Custom(s) => s.as_str(),
            });
            let shape = &type_registry.resolve(type_key).gv_shape;
            let (w, h) = node_dimensions(&node.label);
            dot.push_str(&format!(
                "{}  \"{}\" [label=\"{}\", shape={}, width={:.1}, height={:.1}];\n",
                indent,
                node.id,
                dot_escape(&node.label),
                shape,
                w,
                h
            ));
        }
    }

    dot.push_str(&format!("{}}}\n", indent));
}

fn node_dimensions(label: &str) -> (f64, f64) {
    let char_width = 0.15;
    let min_w = 2.5;
    let w = (label.len() as f64 * char_width + 0.8).max(min_w);
    let h = 1.0;
    (w, h)
}

fn dot_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn run_graphviz(dot_source: &str) -> anyhow::Result<String> {
    let mut child = Command::new("dot")
        .args(["-Tjson0"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to run 'dot' (Graphviz). Is Graphviz installed? Error: {}",
                e
            )
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(dot_source.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Graphviz dot failed: {}", stderr);
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn parse_point(s: &str) -> (f64, f64) {
    let parts: Vec<&str> = s.split(',').collect();
    let x = parts.first().and_then(|p| p.trim().parse().ok()).unwrap_or(0.0);
    let y = parts.get(1).and_then(|p| p.trim().parse().ok()).unwrap_or(0.0);
    (x, y)
}

fn parse_bb(s: &str) -> (f64, f64, f64, f64) {
    let parts: Vec<f64> = s.split(',').filter_map(|p| p.trim().parse().ok()).collect();
    (
        *parts.first().unwrap_or(&0.0),
        *parts.get(1).unwrap_or(&0.0),
        *parts.get(2).unwrap_or(&1920.0),
        *parts.get(3).unwrap_or(&1080.0),
    )
}

/// Convert a Graphviz spline position string to an SVG path.
/// Format: "e,ex,ey sx,sy cx1,cy1 cx2,cy2 ex,ey ..."
fn convert_gv_spline(
    pos: &str,
    scale: f64,
    gv_height: f64,
    offset_x: f64,
    offset_y: f64,
) -> String {
    let transform = |x: f64, y: f64| -> (f64, f64) {
        (x * scale + offset_x, (gv_height - y) * scale + offset_y)
    };

    // Strip endpoint markers
    let cleaned = pos
        .replace("\\", "")
        .replace("\n", " ");

    let mut points: Vec<(f64, f64)> = Vec::new();

    let mut arrow_end: Option<(f64, f64)> = None;

    for token in cleaned.split_whitespace() {
        if token.starts_with("e,") {
            let parts: Vec<f64> = token[2..].split(',').filter_map(|p| p.parse().ok()).collect();
            if parts.len() == 2 {
                arrow_end = Some(transform(parts[0], parts[1]));
            }
            continue;
        }
        if token.starts_with("s,") {
            continue;
        }

        let parts: Vec<f64> = token.split(',').filter_map(|p| p.parse().ok()).collect();
        if parts.len() == 2 {
            points.push(transform(parts[0], parts[1]));
        }
    }

    if points.is_empty() {
        return String::new();
    }

    let mut path = format!("M{:.1},{:.1}", points[0].0, points[0].1);

    // Graphviz splines are sequences of cubic bezier segments (groups of 3 control points)
    let mut i = 1;
    while i + 2 < points.len() {
        path.push_str(&format!(
            " C{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
            points[i].0, points[i].1,
            points[i + 1].0, points[i + 1].1,
            points[i + 2].0, points[i + 2].1
        ));
        i += 3;
    }

    // Handle remaining points as line segments
    while i < points.len() {
        path.push_str(&format!(" L{:.1},{:.1}", points[i].0, points[i].1));
        i += 1;
    }

    // Extend to the arrowhead endpoint if Graphviz provided one
    if let Some((ex, ey)) = arrow_end {
        path.push_str(&format!(" L{:.1},{:.1}", ex, ey));
    }

    path
}

fn compute_group_depths(diagram: &Diagram) -> HashMap<&str, u32> {
    let mut depths: HashMap<&str, u32> = HashMap::new();
    let mut parent_of: HashMap<&str, &str> = HashMap::new();

    for group in &diagram.groups {
        for sub in &group.groups {
            parent_of.insert(sub.as_str(), group.id.as_str());
        }
    }

    for group in &diagram.groups {
        let mut depth = 0u32;
        let mut current = group.id.as_str();
        while let Some(&parent) = parent_of.get(current) {
            depth += 1;
            current = parent;
        }
        depths.insert(group.id.as_str(), depth);
    }

    depths
}
