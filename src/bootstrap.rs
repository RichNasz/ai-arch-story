use std::{
    fs,
    io::{BufRead, Write},
    path::{Path, PathBuf},
};

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
    Repairable {
        missing: Vec<WorkspaceItem>,
    },
    InvalidProjectMetadata {
        error: String,
        missing: Vec<WorkspaceItem>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartChoice {
    Confirm,
    EditWorkspace,
    EditName,
    Quit,
}

pub(crate) fn run_start<R: BufRead, W: Write>(
    workspace: PathBuf,
    name: Option<String>,
    yes: bool,
    input: &mut R,
    output: &mut W,
) -> Result<(), String> {
    let mut workspace = resolve_workspace(&workspace)?;
    let mut name = name.unwrap_or_else(|| default_project_name(&workspace));

    loop {
        writeln!(output, "Workspace: {}", workspace.display())
            .map_err(|error| error.to_string())?;
        writeln!(output, "Project name: {name}").map_err(|error| error.to_string())?;

        match inspect_workspace(&workspace) {
            WorkspaceStatus::InvalidProjectMetadata { error, .. } => {
                writeln!(output, "Cannot initialize workspace: {error}")
                    .map_err(|write_error| write_error.to_string())?;
                return Err(error);
            }
            WorkspaceStatus::Valid => {
                writeln!(output, "Workspace is already valid.")
                    .map_err(|error| error.to_string())?;
                writeln!(output, "{}", serve_guidance(&workspace))
                    .map_err(|error| error.to_string())?;
                return Ok(());
            }
            WorkspaceStatus::Repairable { missing } => {
                if workspace
                    .read_dir()
                    .map_err(|error| error.to_string())?
                    .next()
                    .is_some()
                {
                    writeln!(output, "Existing files will not be changed.")
                        .map_err(|error| error.to_string())?;
                }
                writeln!(output, "Items to add or repair:").map_err(|error| error.to_string())?;
                for item in missing {
                    writeln!(output, "- {}", workspace_item_label(item))
                        .map_err(|error| error.to_string())?;
                }
            }
        }

        if yes {
            break;
        }

        let choice = prompt_choice(input, output)?;
        match choice {
            StartChoice::Confirm => break,
            StartChoice::EditWorkspace => {
                let updated = prompt_value(input, output, "Workspace: ")?;
                workspace = resolve_workspace(Path::new(&updated))?;
                if name.trim().is_empty() {
                    name = default_project_name(&workspace);
                }
            }
            StartChoice::EditName => name = prompt_value(input, output, "Project name: ")?,
            StartChoice::Quit => {
                writeln!(output, "Initialization cancelled.").map_err(|error| error.to_string())?;
                return Ok(());
            }
        }
    }

    if name.trim().is_empty() {
        return Err("project name must be non-empty".to_string());
    }
    initialize_workspace(&workspace, &name)?;
    writeln!(output, "Workspace initialized.").map_err(|error| error.to_string())?;
    writeln!(output, "{}", serve_guidance(&workspace)).map_err(|error| error.to_string())?;
    Ok(())
}

fn initialize_workspace(workspace: &Path, name: &str) -> Result<(), String> {
    let WorkspaceStatus::Repairable { missing } = inspect_workspace(workspace) else {
        return Ok(());
    };

    for item in missing {
        match item {
            WorkspaceItem::ProjectMetadata => {
                let file = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(workspace.join("project.json"))
                    .map_err(|error| format!("could not create project.json: {error}"))?;
                serde_json::to_writer_pretty(file, &ProjectMetadata::new(name))
                    .map_err(|error| format!("could not write project.json: {error}"))?;
            }
            WorkspaceItem::SharedDirectory => fs::create_dir(workspace.join("shared"))
                .map_err(|error| format!("could not create shared/: {error}"))?,
            WorkspaceItem::DiagramsDirectory => fs::create_dir(workspace.join("diagrams"))
                .map_err(|error| format!("could not create diagrams/: {error}"))?,
        }
    }
    Ok(())
}

fn prompt_choice<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
) -> Result<StartChoice, String> {
    loop {
        write!(
            output,
            "Confirm [c], edit workspace [w], edit name [n], or quit [q]: "
        )
        .map_err(|error| error.to_string())?;
        output.flush().map_err(|error| error.to_string())?;
        let mut answer = String::new();
        input
            .read_line(&mut answer)
            .map_err(|error| error.to_string())?;
        match answer.trim().to_ascii_lowercase().as_str() {
            "" | "c" | "confirm" | "y" | "yes" => return Ok(StartChoice::Confirm),
            "w" | "workspace" => return Ok(StartChoice::EditWorkspace),
            "n" | "name" => return Ok(StartChoice::EditName),
            "q" | "quit" => return Ok(StartChoice::Quit),
            _ => writeln!(output, "Please choose confirm, workspace, name, or quit.")
                .map_err(|error| error.to_string())?,
        }
    }
}

fn prompt_value<R: BufRead, W: Write>(
    input: &mut R,
    output: &mut W,
    prompt: &str,
) -> Result<String, String> {
    write!(output, "{prompt}").map_err(|error| error.to_string())?;
    output.flush().map_err(|error| error.to_string())?;
    let mut value = String::new();
    input
        .read_line(&mut value)
        .map_err(|error| error.to_string())?;
    Ok(value.trim().to_string())
}

fn resolve_workspace(workspace: &Path) -> Result<PathBuf, String> {
    let absolute = if workspace.is_absolute() {
        workspace.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| error.to_string())?
            .join(workspace)
    };
    absolute.canonicalize().map_err(|error| {
        format!(
            "could not resolve workspace {}: {error}",
            absolute.display()
        )
    })
}

fn workspace_item_label(item: WorkspaceItem) -> &'static str {
    match item {
        WorkspaceItem::ProjectMetadata => "project.json",
        WorkspaceItem::SharedDirectory => "shared/",
        WorkspaceItem::DiagramsDirectory => "diagrams/",
    }
}

fn serve_guidance(workspace: &Path) -> String {
    format!(
        "Next: ai-arch-story serve --workspace {}",
        workspace.display()
    )
}

pub(crate) fn inspect_workspace(workspace: &Path) -> WorkspaceStatus {
    let project_json = workspace.join("project.json");
    let metadata_error = if project_json.exists() {
        let metadata = match fs::read_to_string(&project_json) {
            Ok(contents) => serde_json::from_str::<ProjectMetadata>(&contents)
                .map_err(|error| format!("project.json is not valid JSON: {error}")),
            Err(error) => Err(format!("could not read project.json: {error}")),
        };

        metadata
            .and_then(|metadata| metadata.validate().map(|()| metadata))
            .err()
    } else {
        None
    };

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

    if let Some(error) = metadata_error {
        WorkspaceStatus::InvalidProjectMetadata { error, missing }
    } else if missing.is_empty() {
        WorkspaceStatus::Valid
    } else {
        WorkspaceStatus::Repairable { missing }
    }
}

pub(crate) fn validate_serve_workspace(workspace: &Path) -> Result<PathBuf, String> {
    let workspace = resolve_workspace(workspace)?;
    let issues = match inspect_workspace(&workspace) {
        WorkspaceStatus::Valid => return Ok(workspace),
        WorkspaceStatus::Repairable { missing } => missing
            .into_iter()
            .map(|item| workspace_item_label(item).to_string())
            .collect(),
        WorkspaceStatus::InvalidProjectMetadata { error, missing } => {
            let mut issues = vec![format!("project.json: {error}")];
            issues.extend(
                missing
                    .into_iter()
                    .map(|item| workspace_item_label(item).to_string()),
            );
            issues
        }
    };

    Err(format!(
        "Cannot start server: workspace is not valid:\n{}\nRepair: ai-arch-story start --workspace {}",
        issues
            .iter()
            .map(|issue| format!("- {issue}"))
            .collect::<Vec<_>>()
            .join("\n"),
        workspace.display()
    ))
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
        io::Cursor,
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::{
        ProjectMetadata, WorkspaceItem, WorkspaceStatus, default_project_name, inspect_workspace,
        run_start, validate_serve_workspace,
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

    #[test]
    fn serve_validation_lists_invalid_metadata_and_all_missing_directories_with_repair_command() {
        // Returning after the metadata error would hide missing directories and permit an unclear repair.
        let workspace = TestWorkspace::new("serve-invalid-workspace");
        fs::write(workspace.root.join("project.json"), "{ not json }")
            .expect("write malformed project metadata");

        let error =
            validate_serve_workspace(&workspace.root).expect_err("reject invalid workspace");
        let resolved = workspace.root.canonicalize().expect("resolve workspace");

        assert!(error.contains("- project.json: project.json is not valid JSON:"));
        assert!(error.contains("- shared/"));
        assert!(error.contains("- diagrams/"));
        assert!(error.contains(&format!(
            "ai-arch-story start --workspace {}",
            resolved.display()
        )));
    }

    #[test]
    fn serve_validation_accepts_an_initialized_workspace() {
        // Rejecting a workspace after bootstrap would make the documented start-to-serve path unusable.
        let workspace = TestWorkspace::new("serve-valid-workspace");
        write_project(&workspace.root, "Serve Valid Workspace");
        fs::create_dir_all(workspace.root.join("shared")).expect("create shared directory");
        fs::create_dir_all(workspace.root.join("diagrams")).expect("create diagrams directory");

        assert_eq!(
            validate_serve_workspace(&workspace.root).expect("accept initialized workspace"),
            workspace.root.canonicalize().expect("resolve workspace")
        );
    }

    #[test]
    fn quitting_interactive_start_leaves_an_empty_workspace_unchanged() {
        // Removing the quit branch would create workspace files and fail this test.
        let workspace = TestWorkspace::new("cancelled-workspace");
        let mut input = Cursor::new(b"q\n");
        let mut output = Vec::new();

        run_start(workspace.root.clone(), None, false, &mut input, &mut output)
            .expect("quit is successful");

        assert_eq!(
            fs::read_dir(&workspace.root)
                .expect("read workspace")
                .count(),
            0
        );
        assert!(
            String::from_utf8(output)
                .expect("utf8 output")
                .contains("Initialization cancelled.")
        );
    }

    #[test]
    fn yes_initializes_empty_workspace_and_prints_exact_serve_guidance() {
        // Omitting any standard workspace item or changing the command must fail this test.
        let workspace = TestWorkspace::new("payments-api");
        let mut input = Cursor::new(Vec::<u8>::new());
        let mut output = Vec::new();

        run_start(workspace.root.clone(), None, true, &mut input, &mut output)
            .expect("initialize workspace");

        assert_eq!(inspect_workspace(&workspace.root), WorkspaceStatus::Valid);
        assert_eq!(
            serde_json::from_str::<ProjectMetadata>(
                &fs::read_to_string(workspace.root.join("project.json")).expect("read project")
            )
            .expect("parse project"),
            ProjectMetadata::new("Payments Api")
        );
        assert!(
            String::from_utf8(output)
                .expect("utf8 output")
                .contains(&format!(
                    "Next: ai-arch-story serve --workspace {}",
                    workspace
                        .root
                        .canonicalize()
                        .expect("resolve workspace")
                        .display()
                ))
        );
    }

    #[test]
    fn yes_repairs_partial_workspace_without_changing_existing_project_or_files() {
        // Rewriting project.json or unrelated files would make this preservation check fail.
        let workspace = TestWorkspace::new("partial-preserved");
        write_project(&workspace.root, "Original Project");
        fs::create_dir_all(workspace.root.join("shared")).expect("create shared directory");
        fs::write(workspace.root.join("keep.txt"), "do not change").expect("write user file");
        let original_project =
            fs::read_to_string(workspace.root.join("project.json")).expect("read project");
        let mut input = Cursor::new(Vec::<u8>::new());
        let mut output = Vec::new();

        run_start(
            workspace.root.clone(),
            Some("Replacement Name".to_string()),
            true,
            &mut input,
            &mut output,
        )
        .expect("repair workspace");

        assert!(workspace.root.join("diagrams").is_dir());
        assert_eq!(
            fs::read_to_string(workspace.root.join("project.json")).expect("read project"),
            original_project
        );
        assert_eq!(
            fs::read_to_string(workspace.root.join("keep.txt")).expect("read user file"),
            "do not change"
        );
        let output = String::from_utf8(output).expect("utf8 output");
        assert!(output.contains("Existing files will not be changed."));
        assert!(output.contains("- diagrams/"));
        assert!(!output.contains("- project.json"));
        assert!(!output.contains("- shared/"));
    }

    #[test]
    fn yes_refuses_invalid_project_json_without_overwriting_it() {
        // Replacing malformed metadata instead of refusing it would fail this test.
        let workspace = TestWorkspace::new("invalid-preserved");
        let invalid = "{ definitely not json }";
        fs::write(workspace.root.join("project.json"), invalid).expect("write invalid project");
        let mut input = Cursor::new(Vec::<u8>::new());
        let mut output = Vec::new();

        assert!(run_start(workspace.root.clone(), None, true, &mut input, &mut output).is_err());

        assert_eq!(
            fs::read_to_string(workspace.root.join("project.json")).expect("read project"),
            invalid
        );
        assert!(!workspace.root.join("shared").exists());
        assert!(!workspace.root.join("diagrams").exists());
    }

    #[test]
    fn interactive_start_allows_editing_workspace_and_name_before_confirmation() {
        // Ignoring either edit response would initialize the wrong location or metadata name.
        let initial = TestWorkspace::new("initial-workspace");
        let selected = TestWorkspace::new("selected-workspace");
        let mut input = Cursor::new(format!(
            "w\n{}\nn\nEdited Project\nc\n",
            selected.root.display()
        ));
        let mut output = Vec::new();

        run_start(initial.root.clone(), None, false, &mut input, &mut output)
            .expect("initialize edited workspace");

        assert_eq!(
            fs::read_dir(&initial.root).expect("read initial").count(),
            0
        );
        assert_eq!(
            serde_json::from_str::<ProjectMetadata>(
                &fs::read_to_string(selected.root.join("project.json")).expect("read project")
            )
            .expect("parse project"),
            ProjectMetadata::new("Edited Project")
        );
    }
}
