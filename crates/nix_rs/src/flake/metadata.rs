use crate::command::{NixCmd, NixCmdError};
use anyhow::Result;
use serde_json::Value;

pub async fn get_flake_metadata_json(cmd: &NixCmd, flake_url: &str) -> Result<Value, NixCmdError> {
    let json = cmd
        .run_with_args_expecting_json::<Value>(&["flake", "metadata", "--json", flake_url])
        .await?;
    Ok(json)
}
