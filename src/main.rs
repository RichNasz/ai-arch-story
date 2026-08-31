mod schema;
mod layout;
mod render;
mod export;
mod bootstrap;
mod workspace;
mod server;

use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ai-arch-story", about = "Generate self-contained HTML architecture diagrams")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize or safely repair a project workspace, then exit
    Start {
        /// Path to the project workspace directory
        #[arg(long, default_value = ".")]
        workspace: PathBuf,

        /// Project display name (defaults to the workspace directory name)
        #[arg(long)]
        name: Option<String>,

        /// Skip interactive confirmation
        #[arg(long)]
        yes: bool,
    },

    /// Render a diagram.json into a self-contained HTML file
    Render {
        /// Path to diagram.json
        input: PathBuf,

        /// Output HTML file path (defaults to output/<name>.html next to input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Start the HTTP server with web editor
    Serve {
        /// Path to the project workspace directory
        #[arg(long, default_value = ".")]
        workspace: PathBuf,

        /// Port to listen on
        #[arg(long, default_value_t = 8080)]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,

        /// Path to webapp static files (for development)
        #[arg(long)]
        static_dir: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { workspace, name, yes } => {
            let stdin = std::io::stdin();
            let stdout = std::io::stdout();
            bootstrap::run_start(workspace, name, yes, &mut stdin.lock(), &mut stdout.lock())
                .map_err(|error| anyhow!(error))
        }
        Commands::Render { input, output } => cmd_render(&input, output.as_deref()),
        Commands::Serve { workspace, port, host, static_dir } => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(server::run_server(workspace, &host, port, static_dir))
        }
    }
}

fn cmd_render(input: &PathBuf, output: Option<&std::path::Path>) -> Result<()> {
    let input_str = fs::read_to_string(input)
        .with_context(|| format!("Failed to read {}", input.display()))?;

    let diagram: schema::Diagram = serde_json::from_str(&input_str)
        .with_context(|| "Failed to parse diagram JSON")?;

    schema::validate_diagram(&diagram)
        .with_context(|| "Diagram validation failed")?;

    let (html, _favicon) = workspace::render_pipeline(input, &diagram)?;

    let output_path = match output {
        Some(p) => p.to_path_buf(),
        None => {
            let parent = input.parent().unwrap_or_else(|| std::path::Path::new("."));
            let stem = input.file_stem().unwrap_or_default().to_string_lossy();
            let name = if stem == "diagram" {
                parent.file_name().unwrap_or_default().to_string_lossy().to_string()
            } else {
                stem.to_string()
            };
            parent.join("output").join(format!("{}.html", name))
        }
    };

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory {}", parent.display()))?;
    }

    fs::write(&output_path, html)
        .with_context(|| format!("Failed to write {}", output_path.display()))?;

    println!("Generated: {}", output_path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Cli, Commands};
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn start_accepts_workspace_name_and_yes_arguments() {
        // Removing any start flag or parsing it as a different command must fail this test.
        let cli = Cli::try_parse_from([
            "ai-arch-story",
            "start",
            "--workspace",
            "demo",
            "--name",
            "Demo Project",
            "--yes",
        ])
        .expect("parse start arguments");

        assert!(
            matches!(cli.command, Commands::Start { workspace, name, yes }
            if workspace == PathBuf::from("demo") && name.as_deref() == Some("Demo Project") && yes)
        );
    }
}
