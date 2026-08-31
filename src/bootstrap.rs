use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ProjectMetadata {
    pub(crate) name: String,
    pub(crate) version: String,
}

impl ProjectMetadata {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "1.0".to_string(),
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("project.json name must be non-empty".to_string());
        }
        if self.version != "1.0" {
            return Err("project.json version must be 1.0".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkspaceItem {
    ProjectMetadata,
    SharedDirectory,
    DiagramsDirectory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WorkspaceStatus {
    Valid,
    Repairable { missing: Vec<WorkspaceItem> },
    InvalidProjectMetadata { error: String },
}

pub(crate) fn inspect_workspace(workspace: &Path) -> WorkspaceStatus {
    let project_json = workspace.join("project.json");
    if project_json.exists() {
        let metadata = match fs::read_to_string(&project_json) {
            Ok(contents) => serde_json::from_str::<ProjectMetadata>(&contents)
                .map_err(|error| format!("project.json is not valid JSON: {error}")),
            Err(error) => Err(format!("could not read project.json: {error}")),
        };

        match metadata.and_then(|metadata| metadata.validate().map(|()| metadata)) {
            Ok(_) => {}
            Err(error) => return WorkspaceStatus::InvalidProjectMetadata { error },
        }
    }

    let mut missing = Vec::new();
    if !project_json.exists() {
        missing.push(WorkspaceItem::ProjectMetadata);
    }
    if !workspace.join("shared").is_dir() {
        missing.push(WorkspaceItem::SharedDirectory);
    }
    if !workspace.join("diagrams").is_dir() {
        missing.push(WorkspaceItem::DiagramsDirectory);
    }

    if missing.is_empty() {
        WorkspaceStatus::Valid
    } else {
        WorkspaceStatus::Repairable { missing }
    }
}

pub(crate) fn default_project_name(workspace: &Path) -> String {
    let basename = workspace
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project");
    let title_cased = basename
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(title_case_word)
        .collect::<Vec<_>>()
        .join(" ");

    if title_cased.is_empty() {
        "Project".to_string()
    } else {
        title_cased
    }
}

fn title_case_word(word: &str) -> String {
    let mut characters = word.chars();
    let Some(first) = characters.next() else {
        return String::new();
    };

    first
        .to_uppercase()
        .chain(characters.flat_map(char::to_lowercase))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::{
        ProjectMetadata, WorkspaceItem, WorkspaceStatus, default_project_name, inspect_workspace,
    };

    static NEXT_WORKSPACE: AtomicUsize = AtomicUsize::new(0);

    struct TestWorkspace {
        root: PathBuf,
        parent: PathBuf,
    }

    impl TestWorkspace {
        fn new(directory_name: &str) -> Self {
            let nonce = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
            let parent = std::env::temp_dir().join(format!(
                "ai-arch-story-bootstrap-tests-{}-{nonce}",
                std::process::id()
            ));
            let root = parent.join(directory_name);
            fs::create_dir_all(&root).expect("create isolated test workspace");
            Self { root, parent }
        }
    }

    impl Drop for TestWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.parent);
        }
    }

    fn write_project(root: &Path, name: &str) {
        fs::write(
            root.join("project.json"),
            serde_json::to_string(&ProjectMetadata::new(name)).expect("serialize project"),
        )
        .expect("write project metadata");
    }

    #[test]
    fn empty_workspace_plans_all_standard_items_and_serializes_metadata() {
        // Removing a bootstrap item or omitting required metadata must make this test fail.
        let workspace = TestWorkspace::new("empty-workspace");

        let status = inspect_workspace(&workspace.root);

        assert_eq!(
            status,
            WorkspaceStatus::Repairable {
                missing: vec![
                    WorkspaceItem::ProjectMetadata,
                    WorkspaceItem::SharedDirectory,
                    WorkspaceItem::DiagramsDirectory,
                ],
            }
        );
        assert_eq!(
            serde_json::to_value(ProjectMetadata::new("Architecture Stories"))
                .expect("serialize project metadata"),
            serde_json::json!({ "name": "Architecture Stories", "version": "1.0" })
        );
    }

    #[test]
    fn default_project_name_title_cases_the_workspace_directory() {
        // Returning the raw directory name would make this test fail.
        let workspace = TestWorkspace::new("edge-payments_api");

        assert_eq!(default_project_name(&workspace.root), "Edge Payments Api");
    }

    #[test]
    fn explicit_name_overrides_the_directory_default() {
        // Ignoring a supplied name would make this metadata choice fail.
        let workspace = TestWorkspace::new("ignored-directory-name");
        let chosen_name = "Payments Modernization";

        let metadata = ProjectMetadata::new(chosen_name);

        assert_ne!(metadata.name, default_project_name(&workspace.root));
        assert_eq!(metadata.name, chosen_name);
    }

    #[test]
    fn valid_workspace_requires_no_repair() {
        // Treating a valid workspace as incomplete would make this test fail.
        let workspace = TestWorkspace::new("valid-workspace");
        write_project(&workspace.root, "Valid Workspace");
        fs::create_dir_all(workspace.root.join("shared")).expect("create shared directory");
        fs::create_dir_all(workspace.root.join("diagrams")).expect("create diagrams directory");

        assert_eq!(inspect_workspace(&workspace.root), WorkspaceStatus::Valid);
    }

    #[test]
    fn partial_workspace_repair_plan_lists_only_missing_items() {
        // Adding existing items to the repair plan would make this test fail.
        let workspace = TestWorkspace::new("partial-workspace");
        write_project(&workspace.root, "Partial Workspace");
        fs::create_dir_all(workspace.root.join("shared")).expect("create shared directory");

        assert_eq!(
            inspect_workspace(&workspace.root),
            WorkspaceStatus::Repairable {
                missing: vec![WorkspaceItem::DiagramsDirectory],
            }
        );
    }

    #[test]
    fn invalid_project_json_refuses_a_repair_plan() {
        // Reclassifying malformed metadata as repairable would permit an unsafe overwrite.
        let workspace = TestWorkspace::new("invalid-project");
        fs::write(workspace.root.join("project.json"), "{ not json }")
            .expect("write malformed project metadata");

        assert!(matches!(
            inspect_workspace(&workspace.root),
            WorkspaceStatus::InvalidProjectMetadata { .. }
        ));
    }
}
