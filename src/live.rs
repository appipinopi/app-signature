use crate::{Creator, Signature};
use anyhow::{Result, ensure};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, RwLock, mpsc},
    thread,
    time::Duration,
};

const ENDPOINT: &str =
    "https://raw.githubusercontent.com/appipinopi/app-signature/main/creator.json";
const LIMIT: u64 = 16 * 1024;
const REFRESH: Duration = Duration::from_secs(3600);

/// Only creator metadata is updated. Executables, scripts and app links are never downloaded.
pub struct LiveSignature {
    current: Arc<RwLock<Signature>>,
    stop: mpsc::Sender<()>,
    worker: Option<thread::JoinHandle<()>>,
}
fn apply(signature: &Signature, json: &[u8]) -> Result<Signature> {
    ensure!(
        json.len() as u64 <= LIMIT,
        "Creator configuration is too large"
    );
    let creator: Creator = serde_json::from_slice(json)?;
    ensure!(
        creator.name.len() <= 120
            && creator.url.len() <= 2048
            && creator
                .works_url
                .as_ref()
                .is_none_or(|url| url.len() <= 2048),
        "Creator fields are too long"
    );
    let updated = Signature {
        creator,
        ..signature.clone()
    };
    updated.validate()?;
    Ok(updated)
}
fn refresh(signature: &Signature, cache: &Path) -> Result<Signature> {
    let client = reqwest::blocking::Client::builder()
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .user_agent("AppSignature/0.2.0")
        .build()?;
    let response = client.get(ENDPOINT).send()?.error_for_status()?;
    let mut bytes = Vec::new();
    response.take(LIMIT + 1).read_to_end(&mut bytes)?;
    let updated = apply(signature, &bytes)?;
    if let Some(parent) = cache.parent() {
        std::fs::create_dir_all(parent)?;
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        file.write_all(&bytes)?;
        file.as_file().sync_all()?;
        file.persist(cache)?;
    }
    Ok(updated)
}
impl LiveSignature {
    /// Immediately uses a valid cached configuration; GitHub refresh is off the UI thread.
    pub fn new(fallback: Signature, cache: PathBuf) -> Self {
        let initial = std::fs::File::open(&cache)
            .ok()
            .and_then(|file| {
                let mut bytes = Vec::new();
                file.take(LIMIT + 1).read_to_end(&mut bytes).ok()?;
                apply(&fallback, &bytes).ok()
            })
            .unwrap_or(fallback);
        let current = Arc::new(RwLock::new(initial));
        let state = current.clone();
        let (stop, receiver) = mpsc::channel();
        let worker = thread::spawn(move || {
            loop {
                let previous = state.read().unwrap().clone();
                if let Ok(updated) = refresh(&previous, &cache) {
                    *state.write().unwrap() = updated;
                }
                if !matches!(
                    receiver.recv_timeout(REFRESH),
                    Err(mpsc::RecvTimeoutError::Timeout)
                ) {
                    break;
                }
            }
        });
        Self {
            current,
            stop,
            worker: Some(worker),
        }
    }
    pub fn snapshot(&self) -> Signature {
        self.current.read().unwrap().clone()
    }
}
impl Drop for LiveSignature {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn remote_creator_updates_works_without_changing_app_links() {
        let fallback = Signature::from_json(r#"{"appName":"Example","links":[{"id":"website","label":"Site","url":"https://example.com"}]}"#).unwrap();
        let updated = apply(&fallback, br#"{"name":"Author","url":"https://github.com/appipinopi","worksUrl":"https://example.org/new"}"#).unwrap();
        assert_eq!(updated.resolve("works").unwrap(), "https://example.org/new");
        assert_eq!(updated.links[0].url, fallback.links[0].url);
        assert!(apply(&fallback, br#"{"name":"Bad","url":"javascript:alert(1)"}"#).is_err());
        assert!(apply(&fallback, &vec![b' '; LIMIT as usize + 1]).is_err());
        assert!(apply(&fallback, b"invalid JSON").is_err());
    }
}
