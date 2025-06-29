use std::process::Command;

use crate::error::GxsyncError;

pub async fn get_access_token(auth_profile: &str) -> Result<String, GxsyncError> {
    let output = Command::new("msoauth")
        .arg("--print-token")
        .arg("--profile")
        .arg(auth_profile)
        .output()
        .map_err(|e| GxsyncError::Auth(format!("failed to run msoauth: {e}")))?;
    if !output.status.success() {
        return Err(GxsyncError::Auth(format!(
            "msoauth failed: {:?}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let token = String::from_utf8(output.stdout)
        .map_err(|e| GxsyncError::Auth(format!("failed to parse msoauth output: {e}")))?
        .lines()
        .next()
        .ok_or_else(|| GxsyncError::Auth("no output from msoauth".to_string()))?
        .trim()
        .to_string();
    Ok(token)
}
