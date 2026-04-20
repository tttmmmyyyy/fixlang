// Per-user trust store recording the user's approvals of `preliminary_commands`.
// Stored as `~/.fixtrust.toml`. See `logs/.../spec.md` for the user-visible specification.

use crate::{
    error::Errors,
    metafiles::project_file::ProjectOrigin,
    misc::warn_msg,
    preliminary_command::PreliminaryCommandMode,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const TRUST_FILE_NAME: &str = ".fixtrust.toml";

// Resolve the default trust-store path (`~/.fixtrust.toml`) for the current user.
// Fails if the home directory cannot be determined.
pub fn default_path() -> Result<PathBuf, Errors> {
    let home = dirs::home_dir().ok_or_else(|| {
        Errors::from_msg("Could not determine home directory for the trust store.".to_string())
    })?;
    Ok(home.join(TRUST_FILE_NAME))
}

// In-memory representation of `~/.fixtrust.toml`: a flat list of approval records
// serialized under the `[[approval]]` TOML array-of-tables.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct TrustStore {
    #[serde(default, rename = "approval")]
    pub approvals: Vec<Approval>,
}

// One approval entry recorded when the user answers `y` at a prompt. The `(source, mode,
// commit_hash?)` triple is the approval key; the remaining fields are informational
// (shown in the TOML file so a human reader can audit what was approved).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Approval {
    // Serialized origin of the project (absolute path for local, `git+<url>` for git).
    pub source: String,
    // `"build"` or `"test"` — which mode's `preliminary_commands` this approval covers.
    pub mode: String,
    // Pinned commit hash for git sources; absent for local/root projects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<String>,
    // RFC-3339 timestamp at which the user granted this approval.
    pub approved_at: String,
    // `[general] name` of the approving project, for human identification.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub project_name: String,
    // Snapshot of the argv list at approval time. Informational only; not used for
    // lookup (commit_hash/source already carry the authoritative identity).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands_preview: Vec<Vec<String>>,
}

impl TrustStore {
    // Load the trust store from the default path (`~/.fixtrust.toml`).
    // A missing file is treated as an empty store.
    // A malformed file emits a warning and is treated as empty so the user gets re-prompted;
    // answering `y` will overwrite the file with a well-formed version.
    pub fn load() -> Result<Self, Errors> {
        let path = default_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to read trust store file `{}`: {}",
                path.display(),
                e
            ))
        })?;
        match toml::from_str::<TrustStore>(&content) {
            Ok(store) => Ok(store),
            Err(e) => {
                warn_msg(&format!(
                    "Failed to parse trust store file `{}`: {}. \
                     Treating it as empty; approvals will be re-prompted.",
                    path.display(),
                    e
                ));
                Ok(Self::default())
            }
        }
    }

    // Check whether an approval entry matching the given (source, mode) tuple exists.
    // For git dependencies the commit hash must also match; for local/root projects only the
    // source (absolute path) and mode are used.
    pub fn is_approved(&self, src: &ProjectOrigin, mode: PreliminaryCommandMode) -> bool {
        let src_key = src.to_trust_key();
        let mode_str = mode.as_str();
        self.approvals.iter().any(|a| {
            a.source == src_key
                && a.mode == mode_str
                && match src {
                    ProjectOrigin::Git { commit, .. } => {
                        a.commit_hash.as_deref() == Some(commit.as_str())
                    }
                    ProjectOrigin::Local(_) => true,
                }
        })
    }

    // Add or replace an approval entry. If an entry with the same (source, mode, commit_hash)
    // already exists it is removed first so the store keeps at most one entry per key.
    pub fn record(&mut self, approval: Approval) {
        self.approvals.retain(|a| {
            !(a.source == approval.source
                && a.mode == approval.mode
                && a.commit_hash == approval.commit_hash)
        });
        self.approvals.push(approval);
    }

    // Save to the default path (`~/.fixtrust.toml`) atomically: write to `<path>.tmp`
    // then rename onto the destination. Returns the path that was written on success.
    pub fn save(&self) -> Result<PathBuf, Errors> {
        let path = default_path()?;
        let content = toml::to_string_pretty(self).map_err(|e| {
            Errors::from_msg(format!("Failed to serialize trust store: {}", e))
        })?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    Errors::from_msg(format!(
                        "Failed to create directory `{}` for trust store: {}",
                        parent.display(),
                        e
                    ))
                })?;
            }
        }
        let tmp_path = {
            let mut p = path.as_os_str().to_os_string();
            p.push(".tmp");
            PathBuf::from(p)
        };
        std::fs::write(&tmp_path, content).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to write temporary trust store file `{}`: {}",
                tmp_path.display(),
                e
            ))
        })?;
        std::fs::rename(&tmp_path, &path).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to rename `{}` to `{}`: {}",
                tmp_path.display(),
                path.display(),
                e
            ))
        })?;
        Ok(path)
    }
}

// Build an `Approval` record from source/mode info and the user-facing metadata.
pub fn make_approval(
    source: &ProjectOrigin,
    mode: PreliminaryCommandMode,
    project_name: String,
    commands_preview: Vec<Vec<String>>,
) -> Approval {
    Approval {
        source: source.to_trust_key(),
        mode: mode.as_str().to_string(),
        commit_hash: source.commit_hash().map(String::from),
        approved_at: chrono::Utc::now().to_rfc3339(),
        project_name,
        commands_preview,
    }
}
