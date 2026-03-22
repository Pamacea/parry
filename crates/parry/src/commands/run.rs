//! Run command - execute a command with file write interception

use oalacea_parry_core::{Config, wrapper::SyncWrapper, wrapper::WrapperConfig, validators::Validators};
use std::path::PathBuf;

/// Run the `parry run` command
pub fn run(
    config: Config,
    command: String,
    args: Vec<String>,
    block: bool,
    _validators: Option<String>,
) -> anyhow::Result<()> {
    let wrapper_config = WrapperConfig {
        block,
        ..Default::default()
    };

    let wrapper = SyncWrapper::new(wrapper_config, config)
        .with_validators(build_validators());

    let exit_code = wrapper.run(&command, &args)?;

    std::process::exit(exit_code);
}

/// Build validators from config
fn build_validators() -> Validators {
    // TODO: Load validators from config
    Validators::new()
}
