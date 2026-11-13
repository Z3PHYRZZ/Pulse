use anyhow::Result;

#[allow(dead_code)]
pub fn notify(title: &str, message: &str) -> Result<()> {
    tracing::info!("notification: {title} — {message}");
    Ok(())
}
