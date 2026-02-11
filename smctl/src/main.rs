use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};

use smctl::{OutputFormat, exit_code, format_output_with};

/// smctl — SmallAIOS control
///
/// Unified CLI and MCP server for the SmallAIOS ecosystem.
/// Manages workspaces, git flow, worktrees, specs, builds, and ModelGate.
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

    /// ModelGate control plane
    Gate {
        #[command(subcommand)]
        command: GateCommands,
    },

    /// Start MCP server
    Serve {
        /// Enable MCP protocol
        #[arg(long)]
        mcp: bool,

        /// Use stdio transport (default)
        #[arg(long)]
        stdio: bool,

        /// Use SSE transport
        #[arg(long)]
        sse: bool,

        /// Use streamable HTTP transport
        #[arg(long)]
        http: bool,

        /// Port for SSE/HTTP transport
        #[arg(long, default_value = "3000")]
        port: u16,
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
        /// Version string
        version: String,
        /// Limit to specific repos
        #[arg(long, value_delimiter = ',')]
        repos: Option<Vec<String>>,
    },
    /// Merge release into main + develop, tag
    Finish {
        /// Version string
        version: String,
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
enum GateCommands {
    /// Check ModelGate health
    Status,
    /// Model management
    Models {
        #[command(subcommand)]
        command: ModelCommands,
    },
    /// Routing table management
    Routes {
        #[command(subcommand)]
        command: RouteCommands,
    },
    /// Run test inference
    Test {
        /// Model name
        model: String,
        /// Input file
        #[arg(long)]
        input: PathBuf,
    },
    /// Stream logs
    Logs {
        /// Follow log output
        #[arg(long)]
        follow: bool,
    },
    /// Security policy management
    Policy {
        #[command(subcommand)]
        command: PolicyCommands,
    },
    /// Trust boundary management
    Boundaries {
        #[command(subcommand)]
        command: BoundaryCommands,
    },
}

#[derive(Subcommand, Debug)]
enum ModelCommands {
    /// List registered models
    List,
    /// Register a new model
    Add {
        /// Path to model file
        path: PathBuf,
    },
    /// Unregister a model
    Remove {
        /// Model name
        name: String,
    },
    /// Verify model tensor shapes against schemas
    Verify {
        /// Model name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
enum RouteCommands {
    /// Show routing table
    List,
    /// Configure a route
    Set {
        /// Model name
        model: String,
        /// Endpoint
        endpoint: String,
    },
}

#[derive(Subcommand, Debug)]
enum PolicyCommands {
    /// Display active SecurityPolicy
    Show,
    /// Author/edit Cedar policies
    Write,
    /// Cedar SMT-based property analysis
    Analyze,
    /// Cedar evaluator: single request check
    Test {
        /// Request JSON file
        request: PathBuf,
    },
    /// Load signed policy blob (ML-DSA-65)
    Load {
        /// Policy blob file
        blob: PathBuf,
    },
    /// Cedar semantic policy comparison
    Diff {
        /// Old policy
        old: PathBuf,
        /// New policy
        new: PathBuf,
    },
    /// Cedar (SMT) + TLA+ (behavioral) verification
    Verify,
    /// Run 5-layer verification on a model
    Check {
        /// Model name
        model: String,
    },
}

#[derive(Subcommand, Debug)]
enum BoundaryCommands {
    /// Show trust boundaries with SecurityLabels
    List,
    /// Verify all crossings have formal proofs
    Check,
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

fn init_tracing(verbose: u8, quiet: bool) {
    let level = if quiet {
        "error"
    } else {
        match verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        }
    };

    let env_filter = std::env::var("SMCTL_LOG").unwrap_or_else(|_| level.to_string());

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .init();
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    init_tracing(cli.verbose, cli.quiet);

    let result = run(cli).await;

    match result {
        Ok(code) => process::exit(code),
        Err(e) => {
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
        smctl_config::find_workspace_root(&cwd).ok_or_else(|| {
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
                    println!("would add repo '{}' ({}) to workspace", repo_name, url);
                    return Ok(exit_code::DRY_RUN);
                }

                smctl_workspace::add_repo(&mut manifest, &repo_name, &url, path.as_deref())?;
                manifest.save_to_root(&root)?;
                println!("added repo '{}' to workspace", repo_name);
                Ok(exit_code::SUCCESS)
            }
            WorkspaceCommands::Remove { repo } => {
                let root = resolve_root()?;
                let mut manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

                if dry_run {
                    println!("would remove repo '{}' from workspace", repo);
                    return Ok(exit_code::DRY_RUN);
                }

                smctl_workspace::remove_repo(&mut manifest, &repo)?;
                manifest.save_to_root(&root)?;
                println!("removed repo '{}' from workspace", repo);
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
                                format!(
                                    "  {:<16} {:<16} {} {}",
                                    s.name,
                                    s.branch,
                                    if s.clean { "\u{2713}" } else { "\u{2717}" },
                                    state
                                )
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
                    println!(
                        "would create worktree set '{}' on branch '{}'",
                        name, branch
                    );
                    return Ok(exit_code::DRY_RUN);
                }

                let infos = smctl_worktree::add_worktree(
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
                let sets = smctl_worktree::list_worktrees(&root, &manifest)?;

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
                    println!("would remove worktree set '{}'", name);
                    return Ok(exit_code::DRY_RUN);
                }

                smctl_worktree::remove_worktree(&root, &manifest, &name, force)?;
                println!("removed worktree set '{}'", name);
                Ok(exit_code::SUCCESS)
            }
            WorktreeCommands::Cd { name } => {
                let root = resolve_root()?;
                let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;
                let path = smctl_worktree::worktree_path(&root, &manifest, &name)?;
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
                                let icon = if rr.success { "\u{2713}" } else { "\u{2717}" };
                                format!("  {} {} — {}", icon, rr.repo_name, rr.message)
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
                        println!("would start feature '{}'", name);
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
                        let _ = smctl_worktree::add_worktree(
                            &root,
                            &manifest,
                            &name,
                            repos.as_deref(),
                            &result.branch_name,
                        );
                        println!("created worktree set '{}'", name);
                    }

                    Ok(exit_code::SUCCESS)
                }
                FeatureCommands::Finish { name } => {
                    let root = resolve_root()?;
                    let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

                    if dry_run {
                        println!("would finish feature '{}'", name);
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
                ReleaseCommands::Start { version, repos } => {
                    let root = resolve_root()?;
                    let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

                    if dry_run {
                        println!("would start release '{}'", version);
                        return Ok(exit_code::DRY_RUN);
                    }

                    let result =
                        smctl_flow::release_start(&root, &manifest, &version, repos.as_deref())?;
                    println!(
                        "{}",
                        format_output_with(&result, fmt, |r| {
                            format!("started release '{}'", r.branch_name)
                        })
                    );
                    Ok(exit_code::SUCCESS)
                }
                ReleaseCommands::Finish { version } => {
                    let root = resolve_root()?;
                    let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

                    if dry_run {
                        println!("would finish release '{}'", version);
                        return Ok(exit_code::DRY_RUN);
                    }

                    let result = smctl_flow::release_finish(&root, &manifest, &version)?;
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
                        println!("would start hotfix '{}'", name);
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
                        println!("would finish hotfix '{}'", name);
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
                        println!("would create spec '{}'", name);
                        return Ok(exit_code::DRY_RUN);
                    }

                    let info = smctl_spec::new_spec(&openspec_dir, &name)?;
                    println!(
                        "{}",
                        format_output_with(&info, fmt, |i| {
                            format!("created spec '{}' at {}", i.name, i.path.display())
                        })
                    );
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
                        println!("would archive spec '{}'", spec_name);
                        return Ok(exit_code::DRY_RUN);
                    }
                    let dest = smctl_spec::archive(&openspec_dir, &spec_name)?;
                    println!("archived spec '{}' to {}", spec_name, dest.display());
                    Ok(exit_code::SUCCESS)
                }
                SpecCommands::Ff { name: _ } => {
                    println!("spec ff: not yet implemented");
                    Ok(exit_code::SUCCESS)
                }
                SpecCommands::Apply { name: _ } => {
                    println!("spec apply: not yet implemented");
                    Ok(exit_code::SUCCESS)
                }
            }
        }

        Commands::Build {
            repo,
            parallel: _,
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

            let report = smctl_build::build(&root, &manifest, repo.as_deref(), test, clean)?;

            println!(
                "{}",
                format_output_with(&report, fmt, |r| {
                    let mut lines: Vec<String> = r
                        .results
                        .iter()
                        .map(|br| {
                            let icon = if br.success { "\u{2713}" } else { "\u{2717}" };
                            format!("  {} {}", icon, br.repo_name)
                        })
                        .collect();
                    if r.all_passed {
                        lines.push(format!("\nbuild passed ({}ms)", r.total_duration_ms));
                    } else {
                        lines.push(format!("\nbuild FAILED ({}ms)", r.total_duration_ms));
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

        Commands::Gate { command } => {
            let gate_client = smctl_gate::GateClient::new(smctl_gate::GateConfig::default());

            match command {
                GateCommands::Status => {
                    let status = gate_client.status().await?;
                    println!(
                        "{}",
                        format_output_with(&status, fmt, |s| {
                            if s.healthy {
                                format!(
                                    "ModelGate at {} — healthy (v{})",
                                    s.url,
                                    s.version.as_deref().unwrap_or("unknown")
                                )
                            } else {
                                format!("ModelGate at {} — unreachable", s.url)
                            }
                        })
                    );
                    Ok(exit_code::SUCCESS)
                }
                GateCommands::Models { command } => match command {
                    ModelCommands::List => {
                        let models = gate_client.models_list().await?;
                        println!(
                            "{}",
                            format_output_with(&models, fmt, |ms| {
                                ms.iter()
                                    .map(|m| {
                                        format!(
                                            "  {} — {} ({})",
                                            m.name,
                                            m.format,
                                            if m.loaded { "loaded" } else { "not loaded" }
                                        )
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            })
                        );
                        Ok(exit_code::SUCCESS)
                    }
                    ModelCommands::Add { path } => {
                        let model = gate_client.models_add(path.to_str().unwrap_or("")).await?;
                        println!("registered model '{}'", model.name);
                        Ok(exit_code::SUCCESS)
                    }
                    ModelCommands::Remove { name } => {
                        gate_client.models_remove(&name).await?;
                        println!("removed model '{}'", name);
                        Ok(exit_code::SUCCESS)
                    }
                    ModelCommands::Verify { name: _ } => {
                        println!("model verify: not yet implemented");
                        Ok(exit_code::SUCCESS)
                    }
                },
                GateCommands::Routes { command } => match command {
                    RouteCommands::List => {
                        let routes = gate_client.routes_list().await?;
                        println!(
                            "{}",
                            format_output_with(&routes, fmt, |rs| {
                                rs.iter()
                                    .map(|r| format!("  {} → {}", r.model, r.endpoint))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            })
                        );
                        Ok(exit_code::SUCCESS)
                    }
                    RouteCommands::Set { model, endpoint } => {
                        gate_client.routes_set(&model, &endpoint).await?;
                        println!("set route {} → {}", model, endpoint);
                        Ok(exit_code::SUCCESS)
                    }
                },
                GateCommands::Test { model, input } => {
                    let input_data: serde_json::Value = {
                        let content =
                            std::fs::read_to_string(&input).context("failed to read input file")?;
                        serde_json::from_str(&content).context("invalid JSON input")?
                    };
                    let result = gate_client.test_inference(&model, &input_data).await?;
                    println!(
                        "{}",
                        format_output_with(&result, fmt, |r| {
                            if r.success {
                                format!("inference OK ({}ms)", r.latency_ms)
                            } else {
                                format!(
                                    "inference FAILED: {}",
                                    r.error.as_deref().unwrap_or("unknown")
                                )
                            }
                        })
                    );
                    Ok(exit_code::SUCCESS)
                }
                GateCommands::Logs { follow: _ } => {
                    println!("gate logs: not yet implemented");
                    Ok(exit_code::SUCCESS)
                }
                GateCommands::Policy { command } => {
                    match command {
                        PolicyCommands::Show => println!("gate policy show: not yet implemented"),
                        PolicyCommands::Write => println!("gate policy write: not yet implemented"),
                        PolicyCommands::Analyze => {
                            println!("gate policy analyze: not yet implemented")
                        }
                        PolicyCommands::Test { .. } => {
                            println!("gate policy test: not yet implemented")
                        }
                        PolicyCommands::Load { .. } => {
                            println!("gate policy load: not yet implemented")
                        }
                        PolicyCommands::Diff { .. } => {
                            println!("gate policy diff: not yet implemented")
                        }
                        PolicyCommands::Verify => {
                            println!("gate policy verify: not yet implemented")
                        }
                        PolicyCommands::Check { .. } => {
                            println!("gate policy check: not yet implemented")
                        }
                    }
                    Ok(exit_code::SUCCESS)
                }
                GateCommands::Boundaries { command } => {
                    match command {
                        BoundaryCommands::List => {
                            println!("gate boundaries list: not yet implemented")
                        }
                        BoundaryCommands::Check => {
                            println!("gate boundaries check: not yet implemented")
                        }
                    }
                    Ok(exit_code::SUCCESS)
                }
            }
        }

        Commands::Serve {
            mcp: _,
            stdio: _,
            sse,
            http,
            port,
        } => {
            let transport = if http {
                smctl_mcp::Transport::Http { port }
            } else if sse {
                smctl_mcp::Transport::Sse { port }
            } else {
                smctl_mcp::Transport::Stdio
            };

            let config = smctl_mcp::McpServerConfig {
                transport,
                workspace_root: cli.workspace,
            };

            smctl_mcp::serve(config).await?;
            Ok(exit_code::SUCCESS)
        }

        Commands::Config { command } => {
            let mut config = smctl_config::SmctlConfig::load_user_config()?;

            match command {
                ConfigCommands::Show => {
                    println!("{}", config.show());
                    Ok(exit_code::SUCCESS)
                }
                ConfigCommands::Get { key } => {
                    match config.get(&key) {
                        Some(value) => println!("{}", value),
                        None => {
                            eprintln!("config key '{}' not set", key);
                            return Ok(exit_code::GENERAL_ERROR);
                        }
                    }
                    Ok(exit_code::SUCCESS)
                }
                ConfigCommands::Set { key, value } => {
                    config.set(&key, &value)?;
                    config.save_user_config()?;
                    println!("set {} = {}", key, value);
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

                    let path = smctl_config::SmctlConfig::user_config_path()?;
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

        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "smctl", &mut std::io::stdout());
            Ok(exit_code::SUCCESS)
        }

        // --- Convenience aliases ---
        Commands::Feat { name } => {
            let root = resolve_root()?;
            let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

            if dry_run {
                println!("would start feature '{}' with worktree", name);
                return Ok(exit_code::DRY_RUN);
            }

            let result = smctl_flow::feature_start(&root, &manifest, &name, None)?;
            let branch = &result.branch_name;
            let _ = smctl_worktree::add_worktree(&root, &manifest, &name, None, branch);
            println!("started feature '{}' with worktree", name);
            Ok(exit_code::SUCCESS)
        }
        Commands::Done { name } => {
            let root = resolve_root()?;
            let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;

            if dry_run {
                println!("would finish feature '{}' and remove worktree", name);
                return Ok(exit_code::DRY_RUN);
            }

            let _ = smctl_worktree::remove_worktree(&root, &manifest, &name, false);
            let _result = smctl_flow::feature_finish(&root, &manifest, &name)?;
            println!("finished feature '{}' and removed worktree", name);
            Ok(exit_code::SUCCESS)
        }
        Commands::Ss { name } => {
            let root = resolve_root()?;
            let manifest = smctl_workspace::WorkspaceManifest::load_from_root(&root)?;
            let openspec_dir = root.join(&manifest.spec.openspec_dir);

            if dry_run {
                println!("would create spec '{}'", name);
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
                println!("build FAILED");
            }
            if report.all_passed {
                Ok(exit_code::SUCCESS)
            } else {
                Ok(exit_code::BUILD_ERROR)
            }
        }
    }
}
