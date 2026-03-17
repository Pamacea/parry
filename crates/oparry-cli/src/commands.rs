//! CLI commands

pub mod check;
pub mod watch;
pub mod wrap;
pub mod init;
pub mod config;
pub mod hook;
pub mod install;

pub use check::CheckCommand;
pub use watch::WatchCommand;
pub use wrap::WrapCommand;
pub use init::InitCommand;
pub use config::ConfigCommand;
pub use hook::HookCommand;
pub use install::InstallCommand;
