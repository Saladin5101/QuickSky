use anyhow::{Result, anyhow};
use lettre::{
    Message, SmtpTransport, Transport,
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
};
use std::fs;
use std::path::Path;
use crate::repo::config::SmtpConfig;

pub fn send_patch(
    smtp: &SmtpConfig,
    to: &str,
    subject: &str,
    patch_path: &Path,
) -> Result<()> {
    let patch_content = fs::read_to_string(patch_path)
        .map_err(|e| anyhow!("Cannot read patch file {}: {}", patch_path.display(), e))?;

    let email = Message::builder()
        .from(smtp.from.parse().map_err(|_| anyhow!("Invalid from address: {}", smtp.from))?)
        .to(to.parse().map_err(|_| anyhow!("Invalid to address: {}", to))?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(patch_content)
        .map_err(|e| anyhow!("Failed to build email: {}", e))?;

    let transport = build_transport(smtp)?;
    transport.send(&email).map_err(|e| anyhow!("Failed to send email: {}", e))?;
    Ok(())
}

fn build_transport(smtp: &SmtpConfig) -> Result<SmtpTransport> {
    let builder = SmtpTransport::relay(&smtp.host)
        .map_err(|e| anyhow!("Invalid SMTP host '{}': {}", smtp.host, e))?
        .port(smtp.port);

    let transport = match &smtp.password {
        Some(pass) => builder
            .credentials(Credentials::new(smtp.from.clone(), pass.clone()))
            .build(),
        None => builder.build(),
    };
    Ok(transport)
}
