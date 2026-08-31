use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub workspace_root: PathBuf,
}

impl AppState {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    pub fn diagrams_dir(&self) -> PathBuf {
        self.workspace_root.join("diagrams")
    }

    pub fn diagram_path(&self, name: &str) -> PathBuf {
        self.diagrams_dir().join(name).join("diagram.json")
    }

    pub fn project_json_path(&self) -> PathBuf {
        self.workspace_root.join("project.json")
    }
}
