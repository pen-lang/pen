use crate::InfrastructureError;
use std::env;

pub fn read(variable: &str) -> Result<String, InfrastructureError> {
    env::var(variable)
        .map_err(|_| InfrastructureError::EnvironmentVariableNotFound(variable.into()))
}
