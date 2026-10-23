// Per-user trust store recording the user's approvals of `preliminary_commands` and `ld_flags`.
// Stored as `~/.fixtrust.toml`. See `logs/.../spec.md` for the user-visible specification.

use crate::{
    build_approval::{BuildRequest, BuildSection},
    error::Errors,
    metafiles::project_file::ProjectOrigin,
    misc::{replace_file, warn_msg},
};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

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
    // `"build"` or `"test"` — which section's `preliminary_commands` and `ld_flags` this approval
    // covers.
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
    /// Snapshot of the `ld_flags` at approval time. Informational only, as `commands_preview` is.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ld_flags_preview: Vec<String>,
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
        let content = fs::read_to_string(&path).map_err(|e| {
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

    /// Checks whether an approval entry covers `request`: one with its source and section. For a
    /// git dependency the entry must also hold the commit the request comes from, and the
    /// `ld_flags` it makes, so that an entry recorded without the user seeing those flags asks
    /// again. For a local/root project the source (absolute path) and section are enough.
    pub fn is_approved(&self, request: &BuildRequest) -> bool {
        let src_key = request.source.to_trust_key();
        let mode_str = request.section.as_str();
        self.approvals.iter().any(|a| {
            a.source == src_key
                && a.mode == mode_str
                && match &request.source {
                    ProjectOrigin::Git { commit, .. } => {
                        a.commit_hash.as_deref() == Some(commit.as_str())
                            && a.ld_flags_preview == request.ld_flags
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

    // Save to the default path (`~/.fixtrust.toml`) atomically, through `replace_file`. Returns
    // the path that was written on success.
    pub fn save(&self) -> Result<PathBuf, Errors> {
        let path = default_path()?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| Errors::from_msg(format!("Failed to serialize trust store: {}", e)))?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    Errors::from_msg(format!(
                        "Failed to create directory `{}` for trust store: {}",
                        parent.display(),
                        e
                    ))
                })?;
            }
        }
        replace_file(&path, |tmp_path| fs::write(tmp_path, &content)).map_err(|e| {
            Errors::from_msg(format!(
                "Failed to write trust store file `{}`: {}",
                path.display(),
                e
            ))
        })?;
        Ok(path)
    }
}

/// Builds the `Approval` record of `request`, stamped with the current time.
pub fn make_approval(request: &BuildRequest) -> Approval {
    Approval {
        source: request.source.to_trust_key(),
        mode: request.section.as_str().to_string(),
        commit_hash: request.source.commit_hash().map(String::from),
        approved_at: chrono::Utc::now().to_rfc3339(),
        project_name: request.project_name.clone(),
        commands_preview: request.preliminary_commands.clone(),
        ld_flags_preview: request.ld_flags.clone(),
    }
}
