//! Shared creator identity, public link validation and system-browser dispatch.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
mod live;
pub use live::LiveSignature;

pub const WEB_SCRIPT: &str = include_str!("../web/app-signature.js");
pub const WEB_STYLE: &str = include_str!("../web/app-signature.css");
pub const CREATOR_JSON: &str = include_str!("../creator.json");

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Creator {
    pub name: String,
    pub url: String,
    #[serde(default, rename = "worksUrl", skip_serializing_if = "Option::is_none")]
    pub works_url: Option<String>,
}
fn default_creator() -> Creator {
    serde_json::from_str(CREATOR_JSON).expect("built-in creator manifest is valid")
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub id: String,
    pub label: String,
    pub url: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Signature {
    pub app_name: String,
    #[serde(default = "default_creator")]
    pub creator: Creator,
    #[serde(default)]
    pub links: Vec<Link>,
}

fn validate_url(value: &str) -> Result<()> {
    ensure!(
        !value.chars().any(char::is_control),
        "Link contains control characters."
    );
    let url = url::Url::parse(value).context("Invalid public URL")?;
    ensure!(
        url.scheme() == "https" && url.host_str().is_some(),
        "Public links must use HTTPS."
    );
    ensure!(
        url.username().is_empty() && url.password().is_none(),
        "Public links cannot contain credentials."
    );
    Ok(())
}
impl Signature {
    pub fn from_json(json: &str) -> Result<Self> {
        let signature: Self = serde_json::from_str(json)?;
        signature.validate()?;
        Ok(signature)
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(!self.app_name.trim().is_empty(), "App name is required.");
        ensure!(
            !self.creator.name.trim().is_empty(),
            "Creator name is required."
        );
        validate_url(&self.creator.url)?;
        if let Some(url) = &self.creator.works_url {
            validate_url(url)?;
        }
        let mut ids = HashSet::from(["creator", "works"]);
        for link in &self.links {
            ensure!(
                !link.id.is_empty()
                    && link
                        .id
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'),
                "Invalid link ID."
            );
            ensure!(
                ids.insert(link.id.as_str()),
                "Duplicate or reserved link ID: {}",
                link.id
            );
            ensure!(!link.label.trim().is_empty(), "Link label is required.");
            validate_url(&link.url)?;
        }
        Ok(())
    }
    /// Resolve registered IDs only. The UI never passes arbitrary URLs to the OS.
    pub fn resolve(&self, id: &str) -> Result<&str> {
        self.validate()?;
        if id == "creator" {
            return Ok(&self.creator.url);
        }
        if id == "works" {
            return self
                .creator
                .works_url
                .as_deref()
                .context("Creator works URL is not configured");
        }
        self.links
            .iter()
            .find(|link| link.id == id)
            .map(|link| link.url.as_str())
            .context("Unknown public link")
    }
    pub fn open(&self, id: &str) -> Result<()> {
        webbrowser::open(self.resolve(id)?).context("Could not open the default browser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_creator_and_registered_links() {
        let signature = Signature::from_json(r#"{"appName":"Example","links":[{"id":"website","label":"公式サイト","url":"https://example.com/"}]}"#).unwrap();
        assert_eq!(signature.creator.name, "appipinopi");
        assert_eq!(
            signature.resolve("works").unwrap(),
            "https://portal.88soundboard.workers.dev/"
        );
        assert_eq!(
            signature.resolve("creator").unwrap(),
            "https://github.com/appipinopi"
        );
        assert_eq!(
            signature.resolve("website").unwrap(),
            "https://example.com/"
        );
        assert!(signature.resolve("https://unregistered.example/").is_err());
    }
    #[test]
    fn rejects_unsafe_urls_and_ambiguous_ids() {
        for url in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "http://example.com/",
            "https://user:pass@example.com/",
            "https://example.com/\n",
        ] {
            assert!(validate_url(url).is_err(), "{url:?}");
        }
        for id in ["creator", "", "bad id"] {
            let json = serde_json::json!({"appName":"Example", "links":[{"id":id,"label":"Site","url":"https://example.com/"}]}).to_string();
            assert!(Signature::from_json(&json).is_err());
        }
        assert!(Signature::from_json(r#"{"appName":"Example","links":[{"id":"site","label":"Site","url":"https://example.com/"},{"id":"site","label":"Other","url":"https://example.org/"}]}"#).is_err());
    }
}
