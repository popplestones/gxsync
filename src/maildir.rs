use crate::error::GxsyncError;
use std::fs;
use std::io::Write;

pub fn write_mail(
    mailbox: &str,
    folder: &str,
    id: &str,
    content: &[u8],
) -> Result<(), GxsyncError> {
    let base = dirs::home_dir()
        .ok_or_else(|| GxsyncError::Other("Could not find home directory".to_string()))?
        .join(".mail")
        .join(mailbox)
        .join(folder);

    for sub in ["cur", "new", "tmp"] {
        fs::create_dir_all(base.join(sub))?;
    }

    let path = base.join("cur").join(format!("{id}.eml"));
    let mut file = fs::File::create(path)?;
    file.write_all(content)?;

    Ok(())
}
