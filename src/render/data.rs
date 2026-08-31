use serde::Serialize;
use std::collections::HashMap;

use crate::layout::LayoutData;
use crate::schema::{Diagram, FlowAnimation, FlowSpeed, ResolvedTypeRegistry};

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedBranding {
    pub organization: Option<String>,
    pub logo_data_uri: Option<String>,
    pub logo_alt: Option<String>,
    pub logo_placement: String,
    pub logo_height: u32,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub footer_text: Option<String>,
    pub show_generated_date: bool,
    pub favicon_data_uri: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DiagramRenderData {
    pub meta: RenderMeta,
    pub layout: LayoutData,
    pub flows: Vec<RenderFlow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branding: Option<RenderBranding>,
    #[serde(rename = "shapeOverrides", skip_serializing_if = "Option::is_none")]
    pub shape_overrides: Option<HashMap<String, String>>,
    #[serde(rename = "typeRegistry", skip_serializing_if = "Option::is_none")]
    pub type_registry: Option<ResolvedTypeRegistry>,
}

#[derive(Debug, Serialize)]
pub struct RenderMeta {
    pub title: String,
    pub description: Option<String>,
    pub theme: String,
    pub generator: String,
}

#[derive(Debug, Serialize)]
pub struct RenderBranding {
    pub organization: Option<String>,
    pub logo_data_uri: Option<String>,
    pub logo_alt: Option<String>,
    pub logo_placement: String,
    pub logo_height: u32,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub footer_text: Option<String>,
    pub show_generated_date: bool,
}

#[derive(Debug, Serialize)]
pub struct RenderFlow {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub color: String,
    pub animation: String,
    pub speed: String,
    pub steps: Vec<RenderFlowStep>,
}

#[derive(Debug, Serialize)]
pub struct RenderFlowStep {
    pub edge: String,
    pub label: Option<String>,
    pub description: Option<String>,
    pub path: String,
    pub from_node: String,
    pub to_node: String,
    pub parallel: bool,
}

const FLOW_COLORS: &[&str] = &["#10B981", "#3B82F6", "#EF4444", "#F59E0B", "#8B5CF6"];

pub fn build_render_data(
    diagram: &Diagram,
    layout: &LayoutData,
    branding: Option<&ResolvedBranding>,
    type_registry: Option<&ResolvedTypeRegistry>,
    shape_overrides: Option<&HashMap<String, String>>,
) -> DiagramRenderData {
    let edge_paths: std::collections::HashMap<&str, &crate::layout::LayoutEdge> = layout
        .edges
        .iter()
        .map(|e| (e.id.as_str(), e))
        .collect();

    let flows: Vec<RenderFlow> = diagram
        .flows
        .iter()
        .enumerate()
        .map(|(i, flow)| {
            let color = flow
                .style
                .as_ref()
                .and_then(|s| s.color.clone())
                .unwrap_or_else(|| FLOW_COLORS[i % FLOW_COLORS.len()].to_string());

            let animation = flow
                .style
                .as_ref()
                .and_then(|s| s.animation)
                .unwrap_or_default();

            let speed = flow
                .style
                .as_ref()
                .and_then(|s| s.speed)
                .unwrap_or_default();

            let steps: Vec<RenderFlowStep> = flow
                .steps
                .iter()
                .map(|step| {
                    let edge = edge_paths.get(step.edge.as_str());
                    RenderFlowStep {
                        edge: step.edge.clone(),
                        label: step.label.clone(),
                        description: step.description.clone(),
                        path: edge.map_or(String::new(), |e| e.path.clone()),
                        from_node: edge.map_or(String::new(), |e| e.from.clone()),
                        to_node: edge.map_or(String::new(), |e| e.to.clone()),
                        parallel: step.parallel.unwrap_or(false),
                    }
                })
                .collect();

            RenderFlow {
                id: flow.id.clone(),
                label: flow.label.clone(),
                description: flow.description.clone(),
                color,
                animation: match animation {
                    FlowAnimation::Particle => "particle".to_string(),
                    FlowAnimation::Pulse => "pulse".to_string(),
                    FlowAnimation::Highlight => "highlight".to_string(),
                },
                speed: match speed {
                    FlowSpeed::Slow => "slow".to_string(),
                    FlowSpeed::Normal => "normal".to_string(),
                    FlowSpeed::Fast => "fast".to_string(),
                },
                steps,
            }
        })
        .collect();

    let render_branding = branding.map(|b| RenderBranding {
        organization: b.organization.clone(),
        logo_data_uri: b.logo_data_uri.clone(),
        logo_alt: b.logo_alt.clone(),
        logo_placement: b.logo_placement.clone(),
        logo_height: b.logo_height,
        primary_color: b.primary_color.clone(),
        secondary_color: b.secondary_color.clone(),
        footer_text: b.footer_text.clone(),
        show_generated_date: b.show_generated_date,
    });

    let so = shape_overrides.filter(|m| !m.is_empty()).cloned();
    let tr = type_registry.cloned();

    DiagramRenderData {
        meta: RenderMeta {
            title: diagram.title.clone(),
            description: diagram.description.clone(),
            theme: diagram.theme.clone(),
            generator: format!("ai-arch-story v{}", env!("CARGO_PKG_VERSION")),
        },
        layout: layout.clone(),
        flows,
        branding: render_branding,
        shape_overrides: so,
        type_registry: tr,
    }
}
