use super::Config;
use crate::error::AptPrepError;
use config::Config as ConfigBuilder;

pub fn load_config(config_path: &str) -> Result<Config, AptPrepError> {
    let config_builder = ConfigBuilder::builder()
        .add_source(config::File::with_name(config_path))
        .build()?;

    config_builder.try_deserialize().map_err(Into::into)
}
