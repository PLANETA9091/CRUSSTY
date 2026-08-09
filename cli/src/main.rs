//! Crussty CLI — server ops for Crussty server builders, module tooling for
//! c-plugin developers.

use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod catalog;
mod init;
mod lib;
mod modules;
mod pack;
mod scaffold;
mod search;
mod server;
mod tui;

#[derive(Parser)]
#[command(name = "crussty", version, about = "CLI for Crussty servers and c-plugins")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Scaffold a Crussty server directory.
    Init(init::InitArgs),
    /// Launch the server; stdin is forwarded to the server console.
    Run,
    /// Stop the running server.
    Stop,
    /// Tail the console log.
    Log { #[arg(long, short)] follow: bool },
    /// List modules and their status (active / parked / disabled).
    Ls,
    /// Activate a parked/disabled module.
    Enable { module: String },
    /// Park (append .x) or disable (--disabled → .disabled) a module.
    Disable {
        module: String,
        #[arg(long)] disabled: bool,
    },
    /// Hot-reload all modules (SIGUSR1 to the running JVM).
    Reload,
    /// Search GitHub for Crussty modules (repos with cplugin.json).
    Search { query: String },
    /// Install a module by id from the catalog, or <owner/repo> directly from GitHub.
    Install {
        module: String,
        /// Catalog repo "owner/name"; defaults to PLANETA9091/crussty-catalog.
        #[arg(long)]
        catalog: Option<String>,
    },
    /// Module developer tooling.
    Module {
        #[command(subcommand)]
        cmd: ModuleCmd,
    },
}

#[derive(Subcommand)]
enum ModuleCmd {
    /// Scaffold a new module from a language template.
    New(scaffold::NewArgs),
    /// Build the module in the current directory.
    Build,
    /// Rebuild and hot-reload on every code change.
    Watch,
    /// Package the module as a distributable tarball.
    Pack(pack::PackArgs),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let code = match cli.command {
        None => {
            if tui::needs_tty() {
                match tui::run() {
                    Some(tui::Action::Run) => server::run(),
                    Some(tui::Action::Stop) => server::stop(),
                    Some(tui::Action::Log) => server::log(false),
                    Some(tui::Action::Ls) => modules::list(),
                    Some(tui::Action::Init { dir }) => init::run(init::InitArgs {
                        dir,
                        version: "1.21.10".into(),
                        release: "v2.0.0".into(),
                        no_kernel: false,
                    }),
                    Some(tui::Action::ModuleNew { name }) => {
                        scaffold::run_new(scaffold::NewArgs {
                            name,
                            template: scaffold::LangTemplate::Rust,
                        })
                    }
                    Some(tui::Action::ModuleBuild) => scaffold::run_build(),
                    Some(tui::Action::ModuleWatch) => scaffold::run_watch(),
                    Some(tui::Action::ModulePack) => {
                        pack::run(pack::PackArgs { version: None, out: None })
                    }
                    Some(tui::Action::Search { query }) => search::search(&query),
                    Some(tui::Action::Install { module }) => {
                        if module.contains('/') {
                            search::install_repo(&module)
                        } else {
                            catalog::install(&module, None)
                        }
                    }
                    Some(tui::Action::Quit) | None => 0,
                }
            } else {
                use clap::CommandFactory;
                let mut cmd = Cli::command();
                let _ = cmd.print_help();
                0
            }
        }
        Some(command) => match command {
            Command::Init(args) => init::run(args),
            Command::Run => server::run(),
            Command::Stop => server::stop(),
            Command::Log { follow } => server::log(follow),
            Command::Ls => modules::list(),
            Command::Enable { module } => modules::set_enabled(&module, true),
            Command::Disable { module, disabled } => {
                if disabled {
                    modules::set_enabled(&module, false)
                } else {
                    modules::park(&module)
                }
            }
            Command::Reload => server::reload(),
            Command::Search { query } => search::search(&query),
            Command::Install { module, catalog } => {
                if module.contains('/') {
                    search::install_repo(&module)
                } else {
                    catalog::install(&module, catalog.as_deref())
                }
            }
            Command::Module { cmd } => match cmd {
                ModuleCmd::New(args) => scaffold::run_new(args),
                ModuleCmd::Build => scaffold::run_build(),
                ModuleCmd::Watch => scaffold::run_watch(),
                ModuleCmd::Pack(args) => pack::run(args),
            },
        },
    };
    ExitCode::from(code as u8)
}