use std::path::PathBuf;
use std::process;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};

use smctl::{OutputFormat, exit_code, format_output_with};

/// smctl — SmallAIOS control
///
/// Unified CLI for the SmallAIOS ecosystem.
/// Manages workspaces, git flow, worktrees, specs, and builds.
#[derive(Parser, Debug)]
#[command(name = "smctl", version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Override workspace root (default: auto-detect from cwd)
    #[arg(short = 'w', long, global = true, env = "SMCTL_WORKSPACE")]
    workspace: Option<PathBuf>,

    /// Increase output verbosity (repeatable: -v, -vv, -vvv)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Suppress non-error output
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Show what would be done without executing
    #[arg(long, global = true)]
    dry_run: bool,

    /// Output in JSON format (for scripting and MCP)
    #[arg(long, global = true)]
    json: bool,

    /// Disable colored output
    #[arg(long, global = true, env = "SMCTL_NO_COLOR")]
    no_color: bool,

    /// Override config file path
    #[arg(short = 'c', long, global = true, env = "SMCTL_CONFIG")]
    config: Option<PathBuf>,

    /// Log level (error, warn, info, debug, trace). Overrides -v/-q.
    #[arg(long, global = true, env = "SMCTL_LOG_LEVEL")]
    log_level: Option<String>,

    /// Write RFC 5424 syslog events to this file (append).
    #[arg(long, global = true, env = "SMCTL_LOG_FILE")]
    log_file: Option<PathBuf>,

    /// Also emit RFC 5424 events to the local syslog Unix socket.
    /// Unix only; on Windows this warns and falls back to stderr.
    #[arg(long, global = true, env = "SMCTL_LOG_SYSLOG")]
    log_syslog: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage multi-repo workspaces
    Workspace {
        #[command(subcommand)]
        command: WorkspaceCommands,
    },

    /// Manage git worktrees for parallel development
    Worktree {
        #[command(subcommand)]
        command: WorktreeCommands,
    },

    /// Git flow branching operations
    Flow {
        #[command(subcommand)]
        command: FlowCommands,
    },

    /// OpenSpec workflow commands
    Spec {
        #[command(subcommand)]
        command: SpecCommands,
    },

    /// Build repos in dependency order
    Build {
        /// Build a specific repo (and its dependencies)
        repo: Option<String>,

        /// Build independent repos concurrently
        #[arg(long)]
        parallel: bool,

        /// Run tests after build
        #[arg(long)]
        test: bool,

        /// Clean before building
        #[arg(long)]
        clean: bool,

        /// Run formal verification (TLA+, Cedar, Lean 4)
        #[arg(long)]
        verify: bool,

        /// Run Cedar policy analysis only (with --verify)
        #[arg(long)]
        cedar: bool,
    },

    /// Run engineering-quality checks across the workspace
    Quality {
        #[command(subcommand)]
        command: QualityCommands,
    },

    /// Talk to a running ModelGate instance (status, models, routes, test, logs)
    Gate {
        /// ModelGate endpoint URL (overrides MODELGATE_URL env and workspace.toml)
        #[arg(long, env = "MODELGATE_URL", global = true)]
        url: Option<String>,

        /// Request timeout in seconds
        #[arg(long, default_value_t = smctl_gate::DEFAULT_TIMEOUT_SECS, global = true)]
        timeout: u64,

        #[command(subcommand)]
        command: GateCommands,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },

    /// Start a long-running service (MCP server, etc.)
    Serve {
        /// Run as an MCP server exposing smctl tools to AI coding assistants
        #[arg(long)]
        mcp: bool,

        /// Use stdio transport (default when --mcp is set without another transport)
        #[arg(long)]
        stdio: bool,

        /// Use SSE / streamable-HTTP transport (local only; mutually exclusive with --stdio)
        #[arg(long)]
        sse: bool,

        /// TCP port for the SSE transport. The default 9377 spells "MCP" on
        /// a phone keypad (M=6, C=2... the digits 9377 are chosen for
        /// memorability and to stay well clear of well-known ports).
        #[arg(long, default_value_t = 9377)]
        port: u16,
    },

    // --- Convenience aliases ---
    /// Start a feature branch + worktree (alias: flow feature start + worktree add)
    Feat {
        /// Feature name
        name: String,
    },

    /// Finish a feature: remove worktree + merge (alias: worktree remove + flow feature finish)
    Done {
        /// Feature name
        name: String,
    },

    /// Create a new OpenSpec feature (alias: spec new)
    Ss {
        /// Spec name
        name: String,
    },

    /// Build all repos (alias: build)
    Sb,
}

#[derive(Subcommand, Debug)]
enum WorkspaceCommands {
    /// Initialize a new workspace
    Init {
        /// Workspace name
        #[arg(long)]
        name: Option<String>,
    },
    /// Add a repo to the workspace
    Add {
        /// Repository URL
        url: String,
        /// Local path for the repo
        #[arg(long)]
        path: Option<String>,
        /// Repository name (default: derived from URL)
        #[arg(long)]
        name: Option<String>,
    },
    /// Remove a repo from the workspace
    Remove {
        /// Repository name
        repo: String,
    },
    /// Show status of all repos
    Status,
    /// Fetch/pull all repos
    Sync,
}

#[derive(Subcommand, Debug)]
enum WorktreeCommands {
    /// Create linked worktrees across repos
    Add {
        /// Worktree set name
        name: String,
        /// Limit to specific repos (comma-separated)
        #[arg(long, value_delimiter = ',')]
        repos: Option<Vec<String>>,
    },
    /// List active worktrees
    List,
    /// Remove a worktree set
    Remove {
        /// Worktree set name
        name: String,
        /// Force removal even with uncommitted changes
        #[arg(long)]
        force: bool,
    },
    /// Print worktree path for shell integration
    Cd {
        /// Worktree set name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
enum FlowCommands {
    /// Initialize git flow in all repos
    Init,
    /// Feature branch operations
    Feature {
        #[command(subcommand)]
        command: FeatureCommands,
    },
    /// Release branch operations
    Release {
        #[command(subcommand)]
        command: ReleaseCommands,
    },
    /// Hotfix branch operations
    Hotfix {
        #[command(subcommand)]
        command: HotfixCommands,
    },
}

#[derive(Subcommand, Debug)]
enum FeatureCommands {
    /// Create a feature branch across repos
    Start {
        /// Feature name
        name: String,
        /// Also create a worktree
        #[arg(long)]
        worktree: bool,
        /// Limit to specific repos
        #[arg(long, value_delimiter = ',')]
        repos: Option<Vec<String>>,
    },
    /// Merge feature into develop
    Finish {
        /// Feature name
        name: String,
    },
    /// List active features
    List,
}

#[derive(Subcommand, Debug)]
enum ReleaseCommands {
    /// Create a release branch from develop
    Start {
        /// Version string (e.g. "1.0.0")
        #[arg(value_name = "VERSION")]
        ver: String,
        /// Limit to specific repos
        #[arg(long, value_delimiter = ',')]
        repos: Option<Vec<String>>,
    },
    /// Merge release into main + develop, tag
    Finish {
        /// Version string (e.g. "1.0.0")
        #[arg(value_name = "VERSION")]
        ver: String,
    },
    /// List active releases
    List,
}

#[derive(Subcommand, Debug)]
enum HotfixCommands {
    /// Create a hotfix from main
    Start {
        /// Hotfix name
        name: String,
        /// Limit to specific repos
        #[arg(long, value_delimiter = ',')]
        repos: Option<Vec<String>>,
    },
    /// Merge hotfix into main + develop
    Finish {
        /// Hotfix name
        name: String,
    },
    /// List active hotfixes
    List,
}

#[derive(Subcommand, Debug)]
enum SpecCommands {
    /// Create a new OpenSpec feature folder
    New {
        /// Spec name
        name: String,
    },
    /// Fast-forward: check document completeness
    Ff {
        /// Spec name (default: current)
        name: Option<String>,
    },
    /// Execute tasks from tasks.md
    Apply {
        /// Spec name (default: current)
        name: Option<String>,
    },
    /// Archive a completed spec
    Archive {
        /// Spec name (default: current)
        name: Option<String>,
    },
    /// Check spec completeness
    Validate {
        /// Spec name (default: current)
        name: Option<String>,
    },
    /// Show spec progress
    Status {
        /// Spec name (default: show all)
        name: Option<String>,
    },
    /// List all specs
    List,
}

#[derive(Subcommand, Debug)]
enum QualityCommands {
    /// Audit dependencies for published RUSTSEC advisories
    Audit {
        /// Emit a machine-readable JSON report on stdout
        #[arg(long)]
        json: bool,

        /// Fail the command when an advisory meets or exceeds this severity
        #[arg(long, default_value = "warning", value_parser = parse_audit_severity)]
        fail_on: smctl_quality::AdvisorySeverity,
    },

    /// Scan for unused dependencies in workspace manifests
    Deps {
        /// Emit a machine-readable JSON report on stdout
        #[arg(long)]
        json: bool,

        /// Fail the command when the unused-dependency count meets this number.
        /// Use 0 to disable the gate and run in report-only mode.
        #[arg(long, default_value_t = 1)]
        fail_on_count: usize,
    },

    /// Report unsafe code usage across the workspace
    Unsafe {
        /// Emit a machine-readable JSON report on stdout
        #[arg(long)]
        json: bool,

        /// Fail the command when the total unsafe-site count meets this number.
        /// Use 0 to disable the gate and run in report-only mode.
        #[arg(long, default_value_t = 0)]
        fail_on_count: u64,
    },

    /// Build a module dependency structure matrix and detect cycles
    Dsm {
        /// Emit a machine-readable JSON report on stdout
        #[arg(long)]
        json: bool,

        /// Fail the command when any module dependency cycle is detected
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        enforce_no_cycles: bool,
    },

    /// Measure cyclomatic and cognitive complexity per function
    Complexity {
        /// Emit a machine-readable JSON report on stdout
        #[arg(long)]
        json: bool,

        /// Fail the command when any function exceeds this cyclomatic complexity
        #[arg(long, default_value_t = 15)]
        cyclomatic_threshold: u32,

        /// Fail the command when any function exceeds this cognitive complexity
        #[arg(long, default_value_t = 25)]
        cognitive_threshold: u32,

        /// Scope the scan to this subdirectory instead of the workspace root
        #[arg(long)]
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum GateCommands {
    /// Show the health and summary metadata of the ModelGate instance
    Status {
        /// Emit a machine-readable JSON report on stdout
        #[arg(long)]
        json: bool,
    },

    /// Manage registered models (list, add, remove)
    Models {
        #[command(subcommand)]
        command: GateModelsCommands,
    },
}

#[derive(Subcommand, Debug)]
enum GateModelsCommands {
    /// List all registered models
    List {
        /// Emit a machine-readable JSON report on stdout
        #[arg(long)]
        json: bool,
    },

    /// Register a new model from a local file
    Add {
        /// Path to the model file (.onnx, .gguf, ...)
        path: PathBuf,

        /// Emit a machine-readable JSON report on stdout
        #[arg(long)]
        json: bool,
    },

    /// Unregister a model by name
    Remove {
        /// Model name
        name: String,

        /// Emit a machine-readable JSON report on stdout
        #[arg(long)]
        json: bool,
    },
}

fn parse_audit_severity(raw: &str) -> Result<smctl_quality::AdvisorySeverity, String> {
    match raw.to_ascii_lowercase().as_str() {
        "none" => Ok(smctl_quality::AdvisorySeverity::None),
        "low" => Ok(smctl_quality::AdvisorySeverity::Low),
        "warning" => Ok(smctl_quality::AdvisorySeverity::Warning),
        "medium" | "moderate" => Ok(smctl_quality::AdvisorySeverity::Medium),
        "high" => Ok(smctl_quality::AdvisorySeverity::High),
        "critical" => Ok(smctl_quality::AdvisorySeverity::Critical),
        other => Err(format!(
            "unknown severity '{other}'. valid values are: none, low, warning, medium, high, critical. pick one of those and pass it to --fail-on"
        )),
    }
}

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    /// Print effective configuration
    Show,
    /// Set a config value
    Set {
        /// Config key (dotted path)
        key: String,
        /// Config value
        value: String,
    },
    /// Get a config value
    Get {
        /// Config key (dotted path)
        key: String,
    },
    /// Open config in editor
    Edit,
}

impl Cli {
    fn output_format(&self) -> OutputFormat {
        if self.json {
            OutputFormat::Json
        } else {
            OutputFormat::Human
        }
    }
}

/// Return true when stdout is attached to a TTY.
///
/// Used by subcommands that default to JSON when their output will be piped
/// into another process (per safety-quality-v1/design.md Decision 9).
fn is_stdout_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

fn human_bytes(n: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut v = n as f64;
    let mut i = 0;
    while v >= 1024.0 && i + 1 < UNITS.len() {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{n} {}", UNITS[0])
    } else {
        format!("{:.1} {}", v, UNITS[i])
    }
}

/// Render a `GateError` to stdout (JSON) or stderr (human + remediation)
/// and return the conventional general-error exit code.
fn gate_error_exit(
    err: smctl_gate::GateError,
    error_kind: &str,
    global_json: bool,
) -> Result<i32> {
    let want_json = global_json || !is_stdout_tty();
    let msg = format!("{err}");
    if want_json {
        let payload = serde_json::json!({
            "error": error_kind,
            "message": msg,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        eprintln!("error: {msg}");
        let hint = match err {
            smctl_gate::GateError::ConnectionRefused { .. } => {
                "start ModelGate and retry, or point smctl at a reachable instance with --url or MODELGATE_URL"
            }
            smctl_gate::GateError::Timeout { .. } => {
                "raise --timeout, or check that ModelGate is responsive"
            }
            smctl_gate::GateError::ModelNotFound { .. } => {
                "run `smctl gate models list` to see registered models"
            }
            smctl_gate::GateError::FileNotFound { .. } => {
                "check the path is correct and the file is readable, then re-run"
            }
            smctl_gate::GateError::HttpError { .. } => {
                "inspect the ModelGate logs for the failing request"
            }
            _ => "re-run with -v for more detail",
        };
        eprintln!("remediation: {hint}");
    }
    Ok(exit_code::GENERAL_ERROR)
}

fn init_tracing(cli: &Cli) -> Result<()> {
    // Precedence (highest first): CLI flags > env vars > workspace.toml
    // [logging] > built-in defaults. CLI + env are fused by clap's
    // `env` attribute, so `cli.log_*` already reflects that layer.
    // Load the manifest optionally — missing workspace is not an
    // error here; we just fall through to defaults.
    let manifest_logging = load_manifest_logging(cli);
    let m = manifest_logging.as_ref();

    let level = if let Some(explicit) = &cli.log_level {
        explicit
            .parse::<smctl_log::LogLevel>()
            .map_err(|e| anyhow::anyhow!("{e}"))?
    } else if cli.quiet {
        smctl_log::LogLevel::Error
    } else if cli.verbose > 0 {
        match cli.verbose {
            1 => smctl_log::LogLevel::Debug,
            _ => smctl_log::LogLevel::Trace,
        }
    } else if let Some(level) = m.and_then(|l| l.level.as_deref()) {
        level
            .parse::<smctl_log::LogLevel>()
            .map_err(|e| anyhow::anyhow!("{e}"))?
    } else {
        smctl_log::LogLevel::Info
    };

    // Resolve transports. CLI/env come first: an explicit --log-file
    // means "file is on"; --log-syslog means "syslog is on". Anything
    // the CLI did not set gets filled from the manifest's transports
    // list, and stderr is on by default if no other transport speaks.
    let file = cli
        .log_file
        .clone()
        .or_else(|| m.and_then(|l| l.file.clone()));

    let manifest_wants = |name: &str| -> bool {
        m.map(|l| l.transports.iter().any(|t| t == name))
            .unwrap_or(false)
    };

    let syslog = cli.log_syslog || manifest_wants("syslog");

    // stderr: on when explicitly listed in the manifest, or by default
    // when no file transport is active. Verbose runs also force it on
    // so interactive users still see events even with --log-file.
    let stderr_from_manifest = manifest_wants("stderr");
    let emit_stderr = if stderr_from_manifest {
        true
    } else if file.is_some() || syslog {
        cli.verbose > 0
    } else {
        true
    };

    let facility = m
        .and_then(|l| l.facility.as_deref())
        .and_then(smctl_workspace::facility_code)
        .unwrap_or(16);

    let config = smctl_log::LoggingConfig {
        level,
        stderr: emit_stderr,
        file,
        syslog,
        facility,
        ..Default::default()
    };

    smctl_log::init(&config).map_err(|e| anyhow::anyhow!("{e}"))
}

/// Best-effort load of the `[logging]` section from `workspace.toml`.
/// Silently returns `None` when there is no workspace, the manifest
/// is unreadable, or the section is absent. Failures here MUST NOT
/// block CLI startup — logging is observability, not a hard
/// dependency.
fn load_manifest_logging(cli: &Cli) -> Option<smctl_workspace::LoggingManifestSection> {
    let root = if let Some(ref path) = cli.workspace {
        path.clone()
    } else {
        let cwd = std::env::current_dir().ok()?;
        smctl::find_workspace_root(&cwd)?
    };
    let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root).ok()?;
    manifest.logging
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // `serve --mcp` owns stdout for the MCP protocol and initializes
    // tracing inside the subcommand handler so it can route logs to
    // stderr via `smctl_log::init(..., stderr: true, file: None)`.
    // Every other command uses the normal init_tracing path.
    let defer_tracing = matches!(cli.command, Commands::Serve { mcp: true, .. });
    if !defer_tracing && let Err(e) = init_tracing(&cli) {
        eprintln!("error: failed to initialize logging: {e:#}");
        process::exit(exit_code::GENERAL_ERROR);
    }

    let result = run(cli).await;

    match result {
        Ok(code) => process::exit(code),
        Err(e) => {
            tracing::error!(
                msgid = %smctl_log::MsgId::Uncategorized,
                error = %e,
                "unhandled error"
            );
            eprintln!("error: {e:#}");
            process::exit(exit_code::GENERAL_ERROR);
        }
    }
}

async fn run(cli: Cli) -> Result<i32> {
    let fmt = cli.output_format();
    let dry_run = cli.dry_run;
    let workspace_override = cli.workspace.clone();

    // Helper closure to resolve workspace root
    let resolve_root = || -> Result<PathBuf> {
        if let Some(ref path) = workspace_override {
            return Ok(path.clone());
        }
        let cwd = std::env::current_dir().context("failed to get current directory")?;
        smctl::find_workspace_root(&cwd).ok_or_else(|| {
            anyhow::anyhow!("no workspace found (use `smctl workspace init` or set --workspace)")
        })
    };

    match cli.command {
        Commands::Workspace { command } => match command {
            WorkspaceCommands::Init { name } => {
                let root = workspace_override
                    .clone()
                    .unwrap_or_else(|| std::env::current_dir().expect("failed to get cwd"));
                let ws_name = name.unwrap_or_else(|| {
                    root.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "workspace".to_string())
                });

                if dry_run {
                    println!(
                        "would initialize workspace '{}' at {}",
                        ws_name,
                        root.display()
                    );
                    return Ok(exit_code::DRY_RUN);
                }

                let manifest = smctl_workspace::init_workspace(&root, &ws_name)?;
                tracing::info!(
                    msgid = %smctl_log::MsgId::WorkspaceInitialized,
                    name = %manifest.workspace.name,
                    path = %root.display(),
                    "initialized workspace"
                );
                println!(
                    "{}",
                    format_output_with(&manifest, fmt, |m| {
                        format!(
                            "initialized workspace '{}' at {}",
                            m.workspace.name,
                            root.display()
                        )
                    })
                );
                Ok(exit_code::SUCCESS)
            }
            WorkspaceCommands::Add { url, path, name } => {
                let root = resolve_root()?;
                let mut manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;
                let repo_name = name.unwrap_or_else(|| {
                    url.rsplit('/')
                        .next()
                        .unwrap_or("repo")
                        .trim_end_matches(".git")
                        .to_string()
                });

                if dry_run {
                    println!("would add repo '{repo_name}' ({url}) to workspace");
                    return Ok(exit_code::DRY_RUN);
                }

                smctl_workspace::add_repo(&mut manifest, &repo_name, &url, path.as_deref())?;
                manifest.save_to_root(&root)?;
                println!("added repo '{repo_name}' to workspace");
                Ok(exit_code::SUCCESS)
            }
            WorkspaceCommands::Remove { repo } => {
                let root = resolve_root()?;
                let mut manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

                if dry_run {
                    println!("would remove repo '{repo}' from workspace");
                    return Ok(exit_code::DRY_RUN);
                }

                smctl_workspace::remove_repo(&mut manifest, &repo)?;
                manifest.save_to_root(&root)?;
                println!("removed repo '{repo}' from workspace");
                Ok(exit_code::SUCCESS)
            }
            WorkspaceCommands::Status => {
                let root = resolve_root()?;
                let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;
                let mut statuses = Vec::new();

                for repo in &manifest.repos {
                    match smctl_workspace::repo_status(&root, repo) {
                        Ok(status) => statuses.push(status),
                        Err(e) => {
                            eprintln!("  {} — error: {}", repo.name, e);
                        }
                    }
                }

                println!(
                    "{}",
                    format_output_with(&statuses, fmt, |ss| {
                        ss.iter()
                            .map(|s| {
                                let state = if s.clean { "clean" } else { "dirty" };
                                format!("  {:<16} {:<16} {}", s.name, s.branch, state)
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                );
                Ok(exit_code::SUCCESS)
            }
            WorkspaceCommands::Sync => {
                let root = resolve_root()?;
                let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

                for repo in &manifest.repos {
                    let repo_path = root.join(repo.local_path());
                    if !repo_path.exists() {
                        eprintln!("  {} — not cloned, skipping", repo.name);
                        continue;
                    }

                    if dry_run {
                        println!("would fetch/pull {}", repo.name);
                        continue;
                    }

                    let result = std::process::Command::new("git")
                        .args(["pull", "--ff-only"])
                        .current_dir(&repo_path)
                        .output();

                    match result {
                        Ok(output) if output.status.success() => {
                            println!("  {} — synced", repo.name);
                        }
                        Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            eprintln!("  {} — failed: {}", repo.name, stderr.trim());
                        }
                        Err(e) => {
                            eprintln!("  {} — error: {}", repo.name, e);
                        }
                    }
                }

                if dry_run {
                    return Ok(exit_code::DRY_RUN);
                }
                Ok(exit_code::SUCCESS)
            }
        },

        Commands::Worktree { command } => match command {
            WorktreeCommands::Add { name, repos } => {
                let root = resolve_root()?;
                let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;
                let branch = format!("{}{}", manifest.flow.feature_prefix, &name);

                if dry_run {
                    println!("would create worktree set '{name}' on branch '{branch}'");
                    return Ok(exit_code::DRY_RUN);
                }

                let infos = smctl_workspace::worktree::add_worktree(
                    &root,
                    &manifest,
                    &name,
                    repos.as_deref(),
                    &branch,
                )?;
                println!(
                    "{}",
                    format_output_with(&infos, fmt, |is| {
                        format!("created worktree set '{}' ({} repos)", name, is.len())
                    })
                );
                Ok(exit_code::SUCCESS)
            }
            WorktreeCommands::List => {
                let root = resolve_root()?;
                let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;
                let sets = smctl_workspace::worktree::list_worktrees(&root, &manifest)?;

                println!(
                    "{}",
                    format_output_with(&sets, fmt, |ss| {
                        if ss.is_empty() {
                            "no active worktrees".to_string()
                        } else {
                            ss.iter()
                                .map(|s| {
                                    let repos: Vec<_> = s
                                        .worktrees
                                        .iter()
                                        .filter(|w| w.exists)
                                        .map(|w| format!("{}@{}", w.repo_name, w.branch))
                                        .collect();
                                    format!("  {} — {}", s.name, repos.join(", "))
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                        }
                    })
                );
                Ok(exit_code::SUCCESS)
            }
            WorktreeCommands::Remove { name, force } => {
                let root = resolve_root()?;
                let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

                if dry_run {
                    println!("would remove worktree set '{name}'");
                    return Ok(exit_code::DRY_RUN);
                }

                smctl_workspace::worktree::remove_worktree(&root, &manifest, &name, force)?;
                println!("removed worktree set '{name}'");
                Ok(exit_code::SUCCESS)
            }
            WorktreeCommands::Cd { name } => {
                let root = resolve_root()?;
                let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;
                let path = smctl_workspace::worktree::worktree_path(&root, &manifest, &name)?;
                // Print path for shell eval: eval "$(smctl worktree cd <name>)"
                println!("{}", path.display());
                Ok(exit_code::SUCCESS)
            }
        },

        Commands::Flow { command } => match command {
            FlowCommands::Init => {
                let root = resolve_root()?;
                let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

                if dry_run {
                    println!(
                        "would initialize git flow in {} repos",
                        manifest.repos.len()
                    );
                    return Ok(exit_code::DRY_RUN);
                }

                let result = smctl_flow::init(&root, &manifest)?;
                println!(
                    "{}",
                    format_output_with(&result, fmt, |r| {
                        r.repos
                            .iter()
                            .map(|rr| {
                                let status = if rr.success { "passed" } else { "failed" };
                                format!("  {} {} — {}", status, rr.repo_name, rr.message)
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                );
                Ok(exit_code::SUCCESS)
            }
            FlowCommands::Feature { command } => match command {
                FeatureCommands::Start {
                    name,
                    worktree,
                    repos,
                } => {
                    let root = resolve_root()?;
                    let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

                    if dry_run {
                        println!("would start feature '{name}'");
                        return Ok(exit_code::DRY_RUN);
                    }

                    let result =
                        smctl_flow::feature_start(&root, &manifest, &name, repos.as_deref())?;
                    println!(
                        "{}",
                        format_output_with(&result, fmt, |r| {
                            format!("started feature '{}'", r.branch_name)
                        })
                    );

                    if worktree {
                        let _ = smctl_workspace::worktree::add_worktree(
                            &root,
                            &manifest,
                            &name,
                            repos.as_deref(),
                            &result.branch_name,
                        );
                        println!("created worktree set '{name}'");
                    }

                    Ok(exit_code::SUCCESS)
                }
                FeatureCommands::Finish { name } => {
                    let root = resolve_root()?;
                    let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

                    if dry_run {
                        println!("would finish feature '{name}'");
                        return Ok(exit_code::DRY_RUN);
                    }

                    let result = smctl_flow::feature_finish(&root, &manifest, &name)?;
                    println!(
                        "{}",
                        format_output_with(&result, fmt, |r| {
                            format!("finished feature '{}'", r.branch_name)
                        })
                    );
                    Ok(exit_code::SUCCESS)
                }
                FeatureCommands::List => {
                    let root = resolve_root()?;
                    let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;
                    let branches = smctl_flow::feature_list(&root, &manifest)?;
                    println!(
                        "{}",
                        format_output_with(&branches, fmt, |bs| {
                            if bs.is_empty() {
                                "no active features".to_string()
                            } else {
                                bs.iter()
                                    .map(|b| format!("  {} — {}", b.repo_name, b.branch))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            }
                        })
                    );
                    Ok(exit_code::SUCCESS)
                }
            },
            FlowCommands::Release { command } => match command {
                ReleaseCommands::Start { ver, repos } => {
                    let root = resolve_root()?;
                    let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

                    if dry_run {
                        println!("would start release '{ver}'");
                        return Ok(exit_code::DRY_RUN);
                    }

                    let result =
                        smctl_flow::release_start(&root, &manifest, &ver, repos.as_deref())?;
                    println!(
                        "{}",
                        format_output_with(&result, fmt, |r| {
                            format!("started release '{}'", r.branch_name)
                        })
                    );
                    Ok(exit_code::SUCCESS)
                }
                ReleaseCommands::Finish { ver } => {
                    let root = resolve_root()?;
                    let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

                    if dry_run {
                        println!("would finish release '{ver}'");
                        return Ok(exit_code::DRY_RUN);
                    }

                    let result = smctl_flow::release_finish(&root, &manifest, &ver)?;
                    println!(
                        "{}",
                        format_output_with(&result, fmt, |r| {
                            format!("finished release '{}'", r.branch_name)
                        })
                    );
                    Ok(exit_code::SUCCESS)
                }
                ReleaseCommands::List => {
                    let root = resolve_root()?;
                    let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;
                    let branches = smctl_flow::release_list(&root, &manifest)?;
                    println!(
                        "{}",
                        format_output_with(&branches, fmt, |bs| {
                            if bs.is_empty() {
                                "no active releases".to_string()
                            } else {
                                bs.iter()
                                    .map(|b| format!("  {} — {}", b.repo_name, b.branch))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            }
                        })
                    );
                    Ok(exit_code::SUCCESS)
                }
            },
            FlowCommands::Hotfix { command } => match command {
                HotfixCommands::Start { name, repos } => {
                    let root = resolve_root()?;
                    let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

                    if dry_run {
                        println!("would start hotfix '{name}'");
                        return Ok(exit_code::DRY_RUN);
                    }

                    let result =
                        smctl_flow::hotfix_start(&root, &manifest, &name, repos.as_deref())?;
                    println!(
                        "{}",
                        format_output_with(&result, fmt, |r| {
                            format!("started hotfix '{}'", r.branch_name)
                        })
                    );
                    Ok(exit_code::SUCCESS)
                }
                HotfixCommands::Finish { name } => {
                    let root = resolve_root()?;
                    let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

                    if dry_run {
                        println!("would finish hotfix '{name}'");
                        return Ok(exit_code::DRY_RUN);
                    }

                    let result = smctl_flow::hotfix_finish(&root, &manifest, &name)?;
                    println!(
                        "{}",
                        format_output_with(&result, fmt, |r| {
                            format!("finished hotfix '{}'", r.branch_name)
                        })
                    );
                    Ok(exit_code::SUCCESS)
                }
                HotfixCommands::List => {
                    let root = resolve_root()?;
                    let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;
                    let branches = smctl_flow::hotfix_list(&root, &manifest)?;
                    println!(
                        "{}",
                        format_output_with(&branches, fmt, |bs| {
                            if bs.is_empty() {
                                "no active hotfixes".to_string()
                            } else {
                                bs.iter()
                                    .map(|b| format!("  {} — {}", b.repo_name, b.branch))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            }
                        })
                    );
                    Ok(exit_code::SUCCESS)
                }
            },
        },

        Commands::Spec { command } => {
            let root = resolve_root()?;
            let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;
            let openspec_dir = root.join(&manifest.spec.openspec_dir);

            match command {
                SpecCommands::New { name } => {
                    if dry_run {
                        println!("would create spec '{name}'");
                        return Ok(exit_code::DRY_RUN);
                    }

                    let info = smctl_spec::new_spec(&openspec_dir, &name)?;
                    tracing::info!(
                        msgid = %smctl_log::MsgId::SpecCreated,
                        name = %info.name,
                        path = %info.path.display(),
                        "spec created"
                    );
                    println!(
                        "{}",
                        format_output_with(&info, fmt, |i| {
                            format!("created spec '{}' at {}", i.name, i.path.display())
                        })
                    );

                    // Auto-create feature branch if workspace is available
                    if let Ok(root) = resolve_root()
                        && let Ok(manifest) =
                            smctl_workspace::WorkspaceManifest::load_from_root(&root)
                    {
                        match smctl_flow::feature_start(&root, &manifest, &name, None) {
                            Ok(result) => {
                                tracing::info!(
                                    msgid = %smctl_log::MsgId::FeatureStarted,
                                    name = %name,
                                    branch = %result.branch_name,
                                    "feature branch created"
                                );
                                println!("created branch '{}'", result.branch_name);
                            }
                            Err(e) => {
                                tracing::warn!(
                                    msgid = %smctl_log::MsgId::Uncategorized,
                                    error = %e,
                                    "could not auto-create branch"
                                );
                            }
                        }
                    }

                    Ok(exit_code::SUCCESS)
                }
                SpecCommands::Validate { name } => {
                    let spec_name = name.context("spec name required")?;
                    let result = smctl_spec::validate(&openspec_dir, &spec_name)?;
                    println!(
                        "{}",
                        format_output_with(&result, fmt, |r| {
                            if r.valid {
                                format!("spec '{}' is valid", r.name)
                            } else {
                                format!(
                                    "spec '{}' has issues:\n{}",
                                    r.name,
                                    r.issues
                                        .iter()
                                        .map(|i| format!("  - {i}"))
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                )
                            }
                        })
                    );
                    if result.valid {
                        Ok(exit_code::SUCCESS)
                    } else {
                        Ok(exit_code::SPEC_ERROR)
                    }
                }
                SpecCommands::Status { name } => {
                    if let Some(name) = name {
                        let info = smctl_spec::spec_info(&openspec_dir, &name)?;
                        println!(
                            "{}",
                            format_output_with(&info, fmt, |i| {
                                format!(
                                    "{}: {:?} [{}/{}]",
                                    i.name, i.phase, i.tasks_done, i.tasks_total
                                )
                            })
                        );
                    } else {
                        let specs = smctl_spec::list_specs(&openspec_dir)?;
                        println!(
                            "{}",
                            format_output_with(&specs, fmt, |ss| {
                                ss.iter()
                                    .map(|s| {
                                        format!(
                                            "  {:<24} {:?}  [{}/{}]",
                                            s.name, s.phase, s.tasks_done, s.tasks_total
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            })
                        );
                    }
                    Ok(exit_code::SUCCESS)
                }
                SpecCommands::List => {
                    let specs = smctl_spec::list_specs(&openspec_dir)?;
                    println!(
                        "{}",
                        format_output_with(&specs, fmt, |ss| {
                            if ss.is_empty() {
                                "no specs found".to_string()
                            } else {
                                ss.iter()
                                    .map(|s| {
                                        format!(
                                            "  {:<24} {:?}  [{}/{}]",
                                            s.name, s.phase, s.tasks_done, s.tasks_total
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            }
                        })
                    );
                    Ok(exit_code::SUCCESS)
                }
                SpecCommands::Archive { name } => {
                    let spec_name = name.context("spec name required")?;
                    if dry_run {
                        println!("would archive spec '{spec_name}'");
                        return Ok(exit_code::DRY_RUN);
                    }
                    let dest = smctl_spec::archive(&openspec_dir, &spec_name)?;
                    tracing::info!(
                        msgid = %smctl_log::MsgId::SpecArchived,
                        name = %spec_name,
                        path = %dest.display(),
                        "spec archived"
                    );
                    println!("archived spec '{}' to {}", spec_name, dest.display());

                    // Auto-finish feature branch if workspace is available
                    if let Ok(root) = resolve_root()
                        && let Ok(manifest) =
                            smctl_workspace::WorkspaceManifest::load_from_root(&root)
                    {
                        match smctl_flow::feature_finish(&root, &manifest, &spec_name) {
                            Ok(result) => {
                                tracing::info!(
                                    msgid = %smctl_log::MsgId::FeatureFinished,
                                    name = %spec_name,
                                    branch = %result.branch_name,
                                    "feature branch merged"
                                );
                                println!("merged branch '{}' into develop", result.branch_name);
                            }
                            Err(e) => {
                                tracing::warn!(
                                    msgid = %smctl_log::MsgId::Uncategorized,
                                    error = %e,
                                    "could not auto-finish branch"
                                );
                            }
                        }
                    }

                    Ok(exit_code::SUCCESS)
                }
                SpecCommands::Ff { name } => {
                    let spec_name = name.context("spec name required")?;

                    // Validate document completeness
                    let result = smctl_spec::validate(&openspec_dir, &spec_name)?;
                    let info = smctl_spec::spec_info(&openspec_dir, &spec_name)?;

                    println!("spec: {spec_name}");
                    println!("phase: {:?}", info.phase);
                    println!(
                        "documents: proposal={} design={} tasks={}",
                        if info.has_proposal {
                            "present"
                        } else {
                            "absent"
                        },
                        if info.has_design { "present" } else { "absent" },
                        if info.has_tasks { "present" } else { "absent" },
                    );
                    println!("tasks: {}/{} complete", info.tasks_done, info.tasks_total);

                    if result.valid {
                        println!("validation: passed");
                        if info.tasks_total > 0 && info.tasks_done == info.tasks_total {
                            println!("ready to archive");
                        } else {
                            println!("{} task(s) remaining", info.tasks_total - info.tasks_done);
                        }
                        Ok(exit_code::SUCCESS)
                    } else {
                        println!("validation: failed");
                        for issue in &result.issues {
                            println!("  - {issue}");
                        }
                        Ok(exit_code::GENERAL_ERROR)
                    }
                }
                SpecCommands::Apply { name } => {
                    let spec_name = name.context("spec name required")?;
                    let info = smctl_spec::spec_info(&openspec_dir, &spec_name)?;

                    if !info.has_tasks {
                        anyhow::bail!(
                            "spec '{spec_name}' has no tasks.md. Apply has no task list to execute. Run `smctl spec ff {spec_name}` to see which documents are missing, then re-scaffold by archiving with `smctl spec archive {spec_name}` and recreating with `smctl spec new {spec_name}`."
                        );
                    }

                    let tasks_path = info.path.join("tasks.md");
                    let content = std::fs::read_to_string(&tasks_path)?;

                    let mut pending = Vec::new();
                    let mut done = Vec::new();
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
                            done.push(
                                trimmed
                                    .trim_start_matches("- [x] ")
                                    .trim_start_matches("- [X] ")
                                    .to_string(),
                            );
                        } else if trimmed.starts_with("- [ ]") {
                            pending.push(trimmed.trim_start_matches("- [ ] ").to_string());
                        }
                    }

                    println!(
                        "spec: {spec_name} — {}/{} tasks complete",
                        done.len(),
                        done.len() + pending.len()
                    );

                    if !pending.is_empty() {
                        println!("\npending ({}):", pending.len());
                        for (i, task) in pending.iter().enumerate() {
                            println!("  {}. {task}", i + 1);
                        }
                    }

                    if pending.is_empty() {
                        println!("all tasks complete — ready for archive");
                    }

                    Ok(exit_code::SUCCESS)
                }
            }
        }

        Commands::Build {
            repo,
            parallel,
            test,
            clean,
            verify: _,
            cedar: _,
        } => {
            let root = resolve_root()?;
            let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

            if dry_run {
                let order = smctl_build::resolve_build_order(&manifest)?;
                let names: Vec<_> = order.iter().map(|r| r.name.as_str()).collect();
                println!("would build in order: {}", names.join(" → "));
                return Ok(exit_code::DRY_RUN);
            }

            tracing::info!(
                msgid = %smctl_log::MsgId::BuildStarted,
                repo = repo.as_deref().unwrap_or("*"),
                parallel = parallel,
                "build started"
            );

            let report = if parallel {
                smctl_build::build_parallel(&root, &manifest, repo.as_deref(), test, clean)?
            } else {
                smctl_build::build(&root, &manifest, repo.as_deref(), test, clean)?
            };

            let passed_count = report.results.iter().filter(|r| r.success).count();
            let failed_count = report.results.len() - passed_count;
            if report.all_passed {
                tracing::info!(
                    msgid = %smctl_log::MsgId::BuildCompleted,
                    repo = repo.as_deref().unwrap_or("*"),
                    duration_ms = report.total_duration_ms as u64,
                    passed_count = passed_count as u64,
                    failed_count = failed_count as u64,
                    "build completed"
                );
            } else {
                let first_failure = report
                    .results
                    .iter()
                    .find(|r| !r.success)
                    .map(|r| r.repo_name.as_str())
                    .unwrap_or("");
                tracing::error!(
                    msgid = %smctl_log::MsgId::BuildFailed,
                    repo = repo.as_deref().unwrap_or("*"),
                    duration_ms = report.total_duration_ms as u64,
                    passed_count = passed_count as u64,
                    failed_count = failed_count as u64,
                    first_failure = first_failure,
                    "build failed"
                );
            }

            println!(
                "{}",
                format_output_with(&report, fmt, |r| {
                    let mut lines: Vec<String> = r
                        .results
                        .iter()
                        .map(|br| {
                            let status = if br.success { "passed" } else { "failed" };
                            format!("  {} {}", status, br.repo_name)
                        })
                        .collect();
                    if r.all_passed {
                        lines.push(format!("\nbuild passed ({}ms)", r.total_duration_ms));
                    } else {
                        lines.push(format!("\nbuild failed ({}ms)", r.total_duration_ms));
                    }
                    lines.join("\n")
                })
            );

            if report.all_passed {
                Ok(exit_code::SUCCESS)
            } else {
                Ok(exit_code::BUILD_ERROR)
            }
        }

        Commands::Quality { command } => match command {
            QualityCommands::Audit { json, fail_on } => {
                // Honour --json on the verb and the global --json, and
                // fall back to JSON when stdout is not a TTY (Decision 9
                // in safety-quality-v1/design.md).
                let want_json = json || cli.json || !is_stdout_tty();

                let root = resolve_root().unwrap_or_else(|_| {
                    std::env::current_dir().expect("failed to get current directory")
                });

                if !smctl_quality::cargo_audit_available() {
                    let msg = "cargo-audit is not installed on PATH. smctl quality audit cannot run without it. Install it with: cargo install cargo-audit";
                    if want_json {
                        let err = serde_json::json!({
                            "error": "tool_missing",
                            "message": msg,
                            "remediation": "cargo install cargo-audit",
                        });
                        println!("{}", serde_json::to_string_pretty(&err)?);
                    } else {
                        eprintln!("error: {msg}");
                    }
                    return Ok(exit_code::GENERAL_ERROR);
                }

                if dry_run {
                    println!("would audit dependencies at {}", root.display());
                    return Ok(exit_code::DRY_RUN);
                }

                let report = match smctl_quality::run_audit(&root) {
                    Ok(r) => smctl_quality::finalise_report(r, fail_on),
                    Err(e) => {
                        if want_json {
                            let err = serde_json::json!({
                                "error": "audit_failed",
                                "message": e.to_string(),
                            });
                            println!("{}", serde_json::to_string_pretty(&err)?);
                        } else {
                            eprintln!("error: {e}");
                        }
                        return Ok(exit_code::GENERAL_ERROR);
                    }
                };

                if want_json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else if report.advisories.is_empty() {
                    println!("audit passed: no advisories found");
                } else {
                    println!(
                        "audit {}: {} advisory/advisories found",
                        if report.fail { "failed" } else { "completed" },
                        report.advisory_count
                    );
                    for a in &report.advisories {
                        println!(
                            "  {} {} ({:?}) installed {} — patched in {}",
                            a.advisory_id,
                            a.crate_name,
                            a.severity,
                            a.installed,
                            if a.patched.is_empty() {
                                "(no fix available)".to_string()
                            } else {
                                a.patched.join(", ")
                            }
                        );
                    }
                    if report.fail {
                        println!(
                            "remediation: run `cargo update -p <crate>` for each affected crate, or upgrade the direct dependency"
                        );
                    }
                }

                if report.fail {
                    Ok(exit_code::GENERAL_ERROR)
                } else {
                    Ok(exit_code::SUCCESS)
                }
            }
            QualityCommands::Deps {
                json,
                fail_on_count,
            } => {
                let want_json = json || cli.json || !is_stdout_tty();

                let root = resolve_root().unwrap_or_else(|_| {
                    std::env::current_dir().expect("failed to get current directory")
                });

                if !smctl_quality::cargo_machete_available() {
                    let msg = "cargo-machete is not installed on PATH. smctl quality deps cannot run without it. Install it with: cargo install cargo-machete";
                    if want_json {
                        let err = serde_json::json!({
                            "error": "tool_missing",
                            "message": msg,
                            "remediation": "cargo install cargo-machete",
                        });
                        println!("{}", serde_json::to_string_pretty(&err)?);
                    } else {
                        eprintln!("error: {msg}");
                    }
                    return Ok(exit_code::GENERAL_ERROR);
                }

                if dry_run {
                    println!("would scan for unused dependencies at {}", root.display());
                    return Ok(exit_code::DRY_RUN);
                }

                let report = match smctl_quality::run_deps(&root) {
                    Ok(r) => smctl_quality::deps::finalise_report(r, fail_on_count),
                    Err(e) => {
                        if want_json {
                            let err = serde_json::json!({
                                "error": "deps_failed",
                                "message": e.to_string(),
                            });
                            println!("{}", serde_json::to_string_pretty(&err)?);
                        } else {
                            eprintln!("error: {e}");
                        }
                        return Ok(exit_code::GENERAL_ERROR);
                    }
                };

                if want_json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else if report.unused.is_empty() {
                    println!("deps passed: no unused dependencies found");
                } else {
                    println!(
                        "deps {}: {} unused dependency/dependencies found",
                        if report.fail { "failed" } else { "completed" },
                        report.unused_count
                    );
                    for u in &report.unused {
                        println!("  {} in {} ({})", u.dependency, u.manifest, u.crate_name);
                    }
                    if report.fail {
                        println!(
                            "remediation: remove each unused entry from the listed Cargo.toml, or add a `package.metadata.cargo-machete` ignore entry if the dependency is a false positive"
                        );
                    }
                }

                if report.fail {
                    Ok(exit_code::GENERAL_ERROR)
                } else {
                    Ok(exit_code::SUCCESS)
                }
            }
            QualityCommands::Unsafe {
                json,
                fail_on_count,
            } => {
                let want_json = json || cli.json || !is_stdout_tty();

                let root = resolve_root().unwrap_or_else(|_| {
                    std::env::current_dir().expect("failed to get current directory")
                });

                if !smctl_quality::cargo_geiger_available() {
                    let msg = "cargo-geiger is not installed on PATH. smctl quality unsafe cannot run without it. Install it with: cargo install cargo-geiger";
                    if want_json {
                        let err = serde_json::json!({
                            "error": "tool_missing",
                            "message": msg,
                            "remediation": "cargo install cargo-geiger",
                        });
                        println!("{}", serde_json::to_string_pretty(&err)?);
                    } else {
                        eprintln!("error: {msg}");
                    }
                    return Ok(exit_code::GENERAL_ERROR);
                }

                if dry_run {
                    println!("would scan unsafe code at {}", root.display());
                    return Ok(exit_code::DRY_RUN);
                }

                let report = match smctl_quality::run_unsafe(&root) {
                    Ok(r) => smctl_quality::unsafe_scan::finalise_report(r, fail_on_count),
                    Err(e) => {
                        if want_json {
                            let err = serde_json::json!({
                                "error": "unsafe_failed",
                                "message": e.to_string(),
                            });
                            println!("{}", serde_json::to_string_pretty(&err)?);
                        } else {
                            eprintln!("error: {e}");
                        }
                        return Ok(exit_code::GENERAL_ERROR);
                    }
                };

                if want_json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else if report.crates.is_empty() {
                    println!("unsafe passed: no unsafe code found");
                } else {
                    println!(
                        "unsafe {}: {} crate(s) with unsafe code, {} site(s) total",
                        if report.fail { "failed" } else { "completed" },
                        report.crate_count,
                        report.total_unsafe
                    );
                    for c in &report.crates {
                        println!(
                            "  {} {} — {} unsafe site(s)",
                            c.crate_name, c.version, c.unsafe_count
                        );
                    }
                    if report.fail {
                        println!(
                            "remediation: review unsafe usage in each listed crate and either refactor to safe code or add a SAFETY: comment justifying each block"
                        );
                    }
                }

                if report.fail {
                    Ok(exit_code::GENERAL_ERROR)
                } else {
                    Ok(exit_code::SUCCESS)
                }
            }
            QualityCommands::Dsm {
                json,
                enforce_no_cycles,
            } => {
                let want_json = json || cli.json || !is_stdout_tty();

                let root = resolve_root().unwrap_or_else(|_| {
                    std::env::current_dir().expect("failed to get current directory")
                });

                if !smctl_quality::cargo_modules_available() {
                    let msg = "cargo-modules is not installed on PATH. smctl quality dsm cannot run without it. Install it with: cargo install cargo-modules";
                    if want_json {
                        let err = serde_json::json!({
                            "error": "tool_missing",
                            "message": msg,
                            "remediation": "cargo install cargo-modules",
                        });
                        println!("{}", serde_json::to_string_pretty(&err)?);
                    } else {
                        eprintln!("error: {msg}");
                    }
                    return Ok(exit_code::GENERAL_ERROR);
                }

                if dry_run {
                    println!("would analyse module structure at {}", root.display());
                    return Ok(exit_code::DRY_RUN);
                }

                let report = match smctl_quality::run_dsm(&root) {
                    Ok(r) => smctl_quality::dsm::finalise_report(r, enforce_no_cycles),
                    Err(e) => {
                        if want_json {
                            let err = serde_json::json!({
                                "error": "dsm_failed",
                                "message": e.to_string(),
                            });
                            println!("{}", serde_json::to_string_pretty(&err)?);
                        } else {
                            eprintln!("error: {e}");
                        }
                        return Ok(exit_code::GENERAL_ERROR);
                    }
                };

                if want_json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else if report.cycles.is_empty() {
                    println!(
                        "dsm passed: no module dependency cycles across {} crate(s)",
                        report.crates_scanned.len()
                    );
                } else {
                    println!(
                        "dsm {}: {} cycle(s) detected across {} crate(s)",
                        if report.fail { "failed" } else { "completed" },
                        report.cycle_count,
                        report.crates_scanned.len()
                    );
                    for c in &report.cycles {
                        println!(
                            "  {}: {} <-> {} ({})",
                            c.crate_name, c.module_a, c.module_b, c.via
                        );
                    }
                    if report.fail {
                        println!(
                            "remediation: break the reported cycle by extracting the shared abstraction into a parent module, or invert the dependency direction"
                        );
                    }
                }

                if report.fail {
                    Ok(exit_code::GENERAL_ERROR)
                } else {
                    Ok(exit_code::SUCCESS)
                }
            }
            QualityCommands::Complexity {
                json,
                cyclomatic_threshold,
                cognitive_threshold,
                path,
            } => {
                let want_json = json || cli.json || !is_stdout_tty();

                let root = path.clone().unwrap_or_else(|| {
                    resolve_root().unwrap_or_else(|_| {
                        std::env::current_dir().expect("failed to get current directory")
                    })
                });

                if !smctl_quality::cargo_rust_code_analysis_available() {
                    let msg = "rust-code-analysis-cli is not installed on PATH. smctl quality complexity cannot run without it. Install it with: cargo install rust-code-analysis-cli";
                    if want_json {
                        let err = serde_json::json!({
                            "error": "tool_missing",
                            "message": msg,
                            "remediation": "cargo install rust-code-analysis-cli",
                        });
                        println!("{}", serde_json::to_string_pretty(&err)?);
                    } else {
                        eprintln!("error: {msg}");
                    }
                    return Ok(exit_code::GENERAL_ERROR);
                }

                if dry_run {
                    println!("would measure complexity at {}", root.display());
                    return Ok(exit_code::DRY_RUN);
                }

                let report = match smctl_quality::run_complexity(&root) {
                    Ok(r) => smctl_quality::complexity::finalise_report(
                        r,
                        cyclomatic_threshold,
                        cognitive_threshold,
                    ),
                    Err(e) => {
                        if want_json {
                            let err = serde_json::json!({
                                "error": "complexity_failed",
                                "message": e.to_string(),
                            });
                            println!("{}", serde_json::to_string_pretty(&err)?);
                        } else {
                            eprintln!("error: {e}");
                        }
                        return Ok(exit_code::GENERAL_ERROR);
                    }
                };

                if want_json {
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else if report.violations.is_empty() {
                    println!(
                        "complexity passed: {} function(s) within thresholds (cyclomatic <= {}, cognitive <= {})",
                        report.function_count,
                        report.threshold_cyclomatic,
                        report.threshold_cognitive,
                    );
                } else {
                    println!(
                        "complexity {}: {} function(s) exceed thresholds (cyclomatic > {}, cognitive > {})",
                        if report.fail { "failed" } else { "completed" },
                        report.violation_count,
                        report.threshold_cyclomatic,
                        report.threshold_cognitive,
                    );
                    for v in &report.violations {
                        println!(
                            "  {} in {}:{}-{} — cyclomatic {}, cognitive {}",
                            v.function, v.file, v.start_line, v.end_line, v.cyclomatic, v.cognitive,
                        );
                    }
                    if report.fail {
                        println!(
                            "remediation: refactor each flagged function into smaller helpers, or add a justification comment with the rationale"
                        );
                    }
                }

                if report.fail {
                    Ok(exit_code::GENERAL_ERROR)
                } else {
                    Ok(exit_code::SUCCESS)
                }
            }
        },

        Commands::Config { command } => {
            let mut config = smctl::SmctlConfig::load_user_config()?;

            match command {
                ConfigCommands::Show => {
                    println!("{}", config.show());
                    Ok(exit_code::SUCCESS)
                }
                ConfigCommands::Get { key } => {
                    match config.get(&key) {
                        Some(value) => println!("{value}"),
                        None => {
                            eprintln!("config key '{key}' not set");
                            return Ok(exit_code::GENERAL_ERROR);
                        }
                    }
                    Ok(exit_code::SUCCESS)
                }
                ConfigCommands::Set { key, value } => {
                    config.set(&key, &value)?;
                    config.save_user_config()?;
                    println!("set {key} = {value}");
                    Ok(exit_code::SUCCESS)
                }
                ConfigCommands::Edit => {
                    let editor = config
                        .user
                        .editor
                        .clone()
                        .or_else(|| std::env::var("SMCTL_EDITOR").ok())
                        .or_else(|| std::env::var("EDITOR").ok())
                        .unwrap_or_else(|| "vi".to_string());

                    let path = smctl::SmctlConfig::user_config_path()?;
                    // Ensure config file exists
                    if !path.exists() {
                        config.save_user_config()?;
                    }
                    let status = std::process::Command::new(&editor)
                        .arg(&path)
                        .status()
                        .context("failed to open editor")?;

                    if status.success() {
                        Ok(exit_code::SUCCESS)
                    } else {
                        Ok(exit_code::GENERAL_ERROR)
                    }
                }
            }
        }

        Commands::Gate {
            url,
            timeout,
            command,
        } => {
            let resolved_url = url.unwrap_or_else(|| smctl_gate::DEFAULT_URL.to_string());
            let cfg = smctl_gate::GateConfig {
                url: resolved_url,
                timeout_secs: timeout,
            };

            let client = match smctl_gate::GateClient::new(cfg.clone()) {
                Ok(c) => c,
                Err(e) => {
                    let want_json = cli.json || !is_stdout_tty();
                    let msg = format!("{e}");
                    if want_json {
                        let err = serde_json::json!({
                            "error": "invalid_config",
                            "message": msg,
                        });
                        println!("{}", serde_json::to_string_pretty(&err)?);
                    } else {
                        eprintln!("error: {msg}");
                        eprintln!(
                            "remediation: pass --url http://<host>:<port> or set MODELGATE_URL to a valid URL"
                        );
                    }
                    return Ok(exit_code::GENERAL_ERROR);
                }
            };

            match command {
                GateCommands::Status { json } => {
                    let want_json = json || cli.json || !is_stdout_tty();

                    if dry_run {
                        println!("would query {} for /health", client.base_url());
                        return Ok(exit_code::DRY_RUN);
                    }

                    match client.health().await {
                        Ok(h) => {
                            if want_json {
                                println!("{}", serde_json::to_string_pretty(&h)?);
                            } else {
                                println!("ModelGate status:");
                                println!("  URL:     {}", client.base_url());
                                println!("  Status:  {}", h.status);
                                println!("  Version: {}", h.version);
                                println!("  Uptime:  {}s", h.uptime_secs);
                                println!("  Models:  {}", h.model_count);
                            }
                            Ok(exit_code::SUCCESS)
                        }
                        Err(e) => gate_error_exit(e, "gate_unreachable", cli.json),
                    }
                }

                GateCommands::Models { command } => match command {
                    GateModelsCommands::List { json } => {
                        let want_json = json || cli.json || !is_stdout_tty();

                        if dry_run {
                            println!("would query {} for /api/v1/models", client.base_url());
                            return Ok(exit_code::DRY_RUN);
                        }

                        match client.list_models().await {
                            Ok(models) => {
                                if want_json {
                                    println!("{}", serde_json::to_string_pretty(&models)?);
                                } else if models.is_empty() {
                                    println!("no models registered");
                                } else {
                                    println!(
                                        "{:<22} {:<8} {:<12} {:<10} REGISTERED",
                                        "NAME", "FORMAT", "SIZE", "STATUS"
                                    );
                                    for m in &models {
                                        println!(
                                            "{:<22} {:<8} {:<12} {:<10} {}",
                                            m.name,
                                            m.format,
                                            human_bytes(m.size_bytes),
                                            m.status,
                                            m.registered_at
                                        );
                                    }
                                }
                                Ok(exit_code::SUCCESS)
                            }
                            Err(e) => gate_error_exit(e, "models_list_failed", cli.json),
                        }
                    }

                    GateModelsCommands::Add { path, json } => {
                        let want_json = json || cli.json || !is_stdout_tty();

                        if !path.exists() {
                            let msg =
                                format!("file not found: {}", path.display());
                            if want_json {
                                let err = serde_json::json!({
                                    "error": "file_not_found",
                                    "message": msg,
                                });
                                println!("{}", serde_json::to_string_pretty(&err)?);
                            } else {
                                eprintln!("error: {msg}");
                                eprintln!(
                                    "remediation: check the path is correct and the file is readable, then re-run"
                                );
                            }
                            return Ok(exit_code::GENERAL_ERROR);
                        }

                        if dry_run {
                            let size =
                                std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                            let name = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("model");
                            println!(
                                "would upload {} ({}) and register as '{}'",
                                path.display(),
                                human_bytes(size),
                                name
                            );
                            return Ok(exit_code::DRY_RUN);
                        }

                        // Progress is printed to stderr so --json stdout stays
                        // a pure machine-readable channel.
                        let progress: Option<smctl_gate::ProgressCallback> = if want_json {
                            None
                        } else {
                            use std::sync::Mutex;
                            let last: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
                            let last_for_cb = last.clone();
                            Some(Arc::new(move |sent: u64, total: Option<u64>| {
                                let mut last_guard = last_for_cb.lock().unwrap();
                                // Throttle updates to once per ~1% so large uploads
                                // don't spam the terminal.
                                let step =
                                    total.map(|t| (t / 100).max(1)).unwrap_or(256 * 1024);
                                if sent - *last_guard >= step || Some(sent) == total {
                                    *last_guard = sent;
                                    match total {
                                        Some(t) => eprint!(
                                            "\ruploading: {} / {} ({:.0}%)",
                                            human_bytes(sent),
                                            human_bytes(t),
                                            (sent as f64 / t as f64) * 100.0
                                        ),
                                        None => eprint!("\ruploading: {}", human_bytes(sent)),
                                    }
                                    use std::io::Write;
                                    let _ = std::io::stderr().flush();
                                }
                            }))
                        };

                        match client.add_model(&path, progress).await {
                            Ok(model) => {
                                if !want_json {
                                    eprintln!();
                                }
                                if want_json {
                                    println!("{}", serde_json::to_string_pretty(&model)?);
                                } else {
                                    println!(
                                        "model '{}' registered ({}, status={})",
                                        model.name,
                                        human_bytes(model.size_bytes),
                                        model.status
                                    );
                                }
                                Ok(exit_code::SUCCESS)
                            }
                            Err(e) => {
                                if !want_json {
                                    eprintln!();
                                }
                                gate_error_exit(e, "models_add_failed", cli.json)
                            }
                        }
                    }

                    GateModelsCommands::Remove { name, json } => {
                        let want_json = json || cli.json || !is_stdout_tty();

                        if dry_run {
                            println!("would unregister model '{name}'");
                            return Ok(exit_code::DRY_RUN);
                        }

                        match client.remove_model(&name).await {
                            Ok(()) => {
                                if want_json {
                                    let ok = serde_json::json!({
                                        "removed": name,
                                    });
                                    println!("{}", serde_json::to_string_pretty(&ok)?);
                                } else {
                                    println!("model '{name}' removed");
                                }
                                Ok(exit_code::SUCCESS)
                            }
                            Err(e) => gate_error_exit(e, "models_remove_failed", cli.json),
                        }
                    }
                },
            }
        }

        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "smctl", &mut std::io::stdout());
            Ok(exit_code::SUCCESS)
        }

        Commands::Serve {
            mcp,
            stdio,
            sse,
            port,
        } => {
            if !mcp {
                anyhow::bail!(
                    "Serve mode not specified. You must choose a server surface. \
                     Run `smctl serve --mcp --stdio` to start the MCP server on stdio."
                );
            }

            if stdio && sse {
                anyhow::bail!(
                    "You specified both --stdio and --sse. \
                     The MCP server binds to exactly one transport per invocation. \
                     Re-run with only one transport flag."
                );
            }

            // Resolve transport. Default to stdio when neither flag is
            // set; `--sse` flips the switch regardless of `--port`
            // (which always has a default). `--port` without `--sse` is
            // silently ignored — the stdio transport has no port.
            let transport = if sse {
                let addr: std::net::SocketAddr = ([127, 0, 0, 1], port).into();
                smctl_mcp::Transport::Sse { addr }
            } else {
                smctl_mcp::Transport::Stdio
            };

            // MCP owns stdout on stdio; SSE routes all protocol bytes
            // over the TCP socket, leaving stdout available. Either way,
            // route tracing to stderr via the shared subscriber.
            // `smctl_log::init` is idempotent and is the single owner of
            // the tracing-subscriber registration
            // (smctl-mcp-v1/design.md Decision 6).
            let level = if cli.quiet {
                smctl_log::LogLevel::Error
            } else {
                match cli.verbose {
                    0 => smctl_log::LogLevel::Info,
                    1 => smctl_log::LogLevel::Debug,
                    _ => smctl_log::LogLevel::Trace,
                }
            };
            let config = smctl_log::LoggingConfig {
                level,
                stderr: true,
                file: cli.log_file.clone(),
                ..Default::default()
            };
            smctl_log::init(&config).map_err(|e| anyhow::anyhow!("{e}"))?;

            let root = resolve_root()?;
            smctl_mcp::start_server(root, transport).await?;
            Ok(exit_code::SUCCESS)
        }

        // --- Convenience aliases ---
        Commands::Feat { name } => {
            let root = resolve_root()?;
            let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

            if dry_run {
                println!("would start feature '{name}' with worktree");
                return Ok(exit_code::DRY_RUN);
            }

            let result = smctl_flow::feature_start(&root, &manifest, &name, None)?;
            let branch = &result.branch_name;
            let _ = smctl_workspace::worktree::add_worktree(&root, &manifest, &name, None, branch);
            println!("started feature '{name}' with worktree");
            Ok(exit_code::SUCCESS)
        }
        Commands::Done { name } => {
            let root = resolve_root()?;
            let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

            if dry_run {
                println!("would finish feature '{name}' and remove worktree");
                return Ok(exit_code::DRY_RUN);
            }

            let _ = smctl_workspace::worktree::remove_worktree(&root, &manifest, &name, false);
            let _result = smctl_flow::feature_finish(&root, &manifest, &name)?;
            println!("finished feature '{name}' and removed worktree");
            Ok(exit_code::SUCCESS)
        }
        Commands::Ss { name } => {
            let root = resolve_root()?;
            let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;
            let openspec_dir = root.join(&manifest.spec.openspec_dir);

            if dry_run {
                println!("would create spec '{name}'");
                return Ok(exit_code::DRY_RUN);
            }

            let info = smctl_spec::new_spec(&openspec_dir, &name)?;
            println!("created spec '{}' at {}", info.name, info.path.display());
            Ok(exit_code::SUCCESS)
        }
        Commands::Sb => {
            let root = resolve_root()?;
            let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

            if dry_run {
                println!("would build all repos");
                return Ok(exit_code::DRY_RUN);
            }

            let report = smctl_build::build(&root, &manifest, None, false, false)?;
            if report.all_passed {
                println!("build passed");
            } else {
                println!("build failed");
            }
            if report.all_passed {
                Ok(exit_code::SUCCESS)
            } else {
                Ok(exit_code::BUILD_ERROR)
            }
        }
    }
}
