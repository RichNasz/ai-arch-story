use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagram {
    pub version: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub viewport: Option<Viewport>,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub flows: Vec<Flow>,
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(default)]
    pub branding: Option<DiagramBranding>,
    #[serde(default)]
    pub custom_types: Option<CustomTypes>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramBranding {
    #[serde(default = "branding_enabled_default")]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub organization: Option<String>,
    #[serde(default)]
    pub logo: Option<BrandingLogo>,
    #[serde(default)]
    pub colors: Option<BrandingColors>,
    #[serde(default)]
    pub footer: Option<BrandingFooter>,
    #[serde(default)]
    pub favicon: Option<BrandingFavicon>,
}

fn branding_enabled_default() -> Option<bool> {
    None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingLogo {
    pub src: String,
    #[serde(default)]
    pub alt: Option<String>,
    #[serde(default)]
    pub placement: Option<String>,
    #[serde(default)]
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingColors {
    #[serde(default)]
    pub primary: Option<String>,
    #[serde(default)]
    pub secondary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingFooter {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(rename = "showGeneratedDate", default)]
    pub show_generated_date: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingFavicon {
    pub src: String,
}

fn default_theme() -> String {
    "default".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    #[serde(default, rename = "type")]
    pub node_type: Option<NodeType>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub style: Option<StyleOverride>,
    #[serde(default)]
    pub position: Option<Position>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    BuiltIn(BuiltInNodeType),
    Custom(String),
}

impl NodeType {
    pub fn as_key(&self) -> String {
        match self {
            NodeType::BuiltIn(b) => b.as_key().to_string(),
            NodeType::Custom(s) => s.clone(),
        }
    }
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::BuiltIn(BuiltInNodeType::Generic)
    }
}

impl Serialize for NodeType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.as_key())
    }
}

impl<'de> Deserialize<'de> for NodeType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match BuiltInNodeType::from_str(&s) {
            Some(b) => Ok(NodeType::BuiltIn(b)),
            None => Ok(NodeType::Custom(s)),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BuiltInNodeType {
    Service,
    Datastore,
    Queue,
    User,
    External,
    Function,
    Gateway,
    Frontend,
    Storage,
    Generic,
}

impl BuiltInNodeType {
    pub fn as_key(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Datastore => "datastore",
            Self::Queue => "queue",
            Self::User => "user",
            Self::External => "external",
            Self::Function => "function",
            Self::Gateway => "gateway",
            Self::Frontend => "frontend",
            Self::Storage => "storage",
            Self::Generic => "generic",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "service" => Some(Self::Service),
            "datastore" => Some(Self::Datastore),
            "queue" => Some(Self::Queue),
            "user" => Some(Self::User),
            "external" => Some(Self::External),
            "function" => Some(Self::Function),
            "gateway" => Some(Self::Gateway),
            "frontend" => Some(Self::Frontend),
            "storage" => Some(Self::Storage),
            "generic" => Some(Self::Generic),
            _ => None,
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            Self::Service, Self::Datastore, Self::Queue, Self::User,
            Self::External, Self::Function, Self::Gateway, Self::Frontend,
            Self::Storage, Self::Generic,
        ]
    }

    pub fn default_shape(&self) -> &'static str {
        match self {
            Self::Service => "rounded-rect",
            Self::Datastore => "cylinder",
            Self::Queue => "parallelogram",
            Self::User => "person",
            Self::External => "dashed-rect",
            Self::Function => "hexagon",
            Self::Gateway => "diamond",
            Self::Frontend => "browser",
            Self::Storage => "folder",
            Self::Generic => "rect",
        }
    }

    pub fn accent_color(&self) -> &'static str {
        match self {
            Self::Service => "#3B82F6",
            Self::Datastore => "#8B5CF6",
            Self::Queue => "#F59E0B",
            Self::Gateway => "#06B6D4",
            Self::Frontend => "#10B981",
            Self::External => "#6B7280",
            Self::Function => "#EC4899",
            Self::User => "#14B8A6",
            Self::Storage => "#A78BFA",
            Self::Generic => "#94A3B8",
        }
    }

    pub fn default_gv_shape(&self) -> &'static str {
        match self {
            Self::Service => "box",
            Self::Datastore => "cylinder",
            Self::Queue => "parallelogram",
            Self::User => "house",
            Self::External => "box",
            Self::Function => "hexagon",
            Self::Gateway => "diamond",
            Self::Frontend => "box",
            Self::Storage => "folder",
            Self::Generic => "box",
        }
    }
}

// --- Custom Type Definitions ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTypes {
    #[serde(default)]
    pub types: HashMap<String, CustomTypeDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTypeDef {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub shape: Option<String>,
    #[serde(default, rename = "gvShape")]
    pub gv_shape: Option<String>,
    #[serde(default, rename = "accentColor")]
    pub accent_color: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

// --- Resolved Type Registry ---

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedType {
    pub key: String,
    pub label: String,
    pub shape: String,
    pub gv_shape: String,
    pub accent_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub is_built_in: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedTypeRegistry {
    pub types: HashMap<String, ResolvedType>,
}

impl ResolvedTypeRegistry {
    pub fn built_in() -> Self {
        let mut types = HashMap::new();
        for bt in BuiltInNodeType::all() {
            let key = bt.as_key().to_string();
            types.insert(key.clone(), ResolvedType {
                key: key.clone(),
                label: capitalize(&key),
                shape: bt.default_shape().to_string(),
                gv_shape: bt.default_gv_shape().to_string(),
                accent_color: bt.accent_color().to_string(),
                description: None,
                is_built_in: true,
            });
        }
        Self { types }
    }

    pub fn merge(&mut self, custom: &CustomTypes) {
        for (key, def) in &custom.types {
            if let Some(existing) = self.types.get_mut(key) {
                if let Some(ref shape) = def.shape {
                    existing.shape = shape.clone();
                }
                if let Some(ref gv) = def.gv_shape {
                    existing.gv_shape = gv.clone();
                }
                if let Some(ref color) = def.accent_color {
                    existing.accent_color = color.clone();
                }
                if let Some(ref desc) = def.description {
                    existing.description = Some(desc.clone());
                }
                if let Some(ref label) = def.label {
                    existing.label = label.clone();
                }
            } else {
                let shape = def.shape.as_deref().unwrap_or("rounded-rect");
                let gv = def.gv_shape.clone().unwrap_or_else(|| shape_to_gv_shape(shape).to_string());
                self.types.insert(key.clone(), ResolvedType {
                    key: key.clone(),
                    label: def.label.clone().unwrap_or_else(|| capitalize(key)),
                    shape: shape.to_string(),
                    gv_shape: gv,
                    accent_color: def.accent_color.clone().unwrap_or_else(|| "#94A3B8".to_string()),
                    description: def.description.clone(),
                    is_built_in: false,
                });
            }
        }
    }

    pub fn resolve(&self, key: &str) -> &ResolvedType {
        self.types.get(key).unwrap_or_else(|| self.types.get("generic").unwrap())
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn shape_to_gv_shape(shape: &str) -> &str {
    match shape {
        "rounded-rect" => "box",
        "cylinder" => "cylinder",
        "parallelogram" => "parallelogram",
        "person" => "house",
        "dashed-rect" => "box",
        "hexagon" => "hexagon",
        "diamond" => "diamond",
        "browser" => "box",
        "folder" => "folder",
        "rect" => "box",
        _ => "box",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub direction: Option<EdgeDirection>,
    #[serde(default)]
    pub style: Option<StyleOverride>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeDirection {
    OneWay,
    TwoWay,
    None,
}

impl Default for EdgeDirection {
    fn default() -> Self {
        EdgeDirection::OneWay
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flow {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    pub steps: Vec<FlowStep>,
    #[serde(default)]
    pub style: Option<FlowStyle>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    pub edge: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parallel: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStyle {
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub speed: Option<FlowSpeed>,
    #[serde(default)]
    pub animation: Option<FlowAnimation>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FlowSpeed {
    Slow,
    Normal,
    Fast,
}

impl Default for FlowSpeed {
    fn default() -> Self {
        FlowSpeed::Normal
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FlowAnimation {
    Pulse,
    Particle,
    Highlight,
}

impl Default for FlowAnimation {
    fn default() -> Self {
        FlowAnimation::Highlight
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub label: String,
    pub nodes: Vec<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub style: Option<StyleOverride>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleOverride {
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub border: Option<String>,
    #[serde(default)]
    pub opacity: Option<f64>,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default)]
    pub shape: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Viewport {
    #[serde(default = "default_viewport_width")]
    pub width: f64,
    #[serde(default = "default_viewport_height")]
    pub height: f64,
}

fn default_viewport_width() -> f64 {
    1920.0
}

fn default_viewport_height() -> f64 {
    1080.0
}

impl Default for Viewport {
    fn default() -> Self {
        Viewport {
            width: 1920.0,
            height: 1080.0,
        }
    }
}
