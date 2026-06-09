//! # spec-spine-core
//!
//! The spec-spine engine. Phase 2 ships **compile** (markdown corpus →
//! deterministic `registry.json`) and **query** (typed read-only access over a
//! loaded registry). `index` / `lint` / `couple` / `init` land in later phases.
//!
//! Every artifact-producing function is a pure function of `(config, file
//! contents)` — no ambient clock or environment reads. The public API returns
//! owned, `serde`-serializable DTOs (from [`spec_spine_types`]); the
//! JSON-in/JSON-out facade ([`compile_json`], [`query_json`], [`load_config_json`])
//! is the seam future FFI bindings wrap.

mod canonical_json;
pub mod compile;
mod hash;
mod markdown;
pub mod query;

use serde::Deserialize;
use spec_spine_types::{Config, Error, Status, load_config};

// Re-export the type substrate so callers depend on one crate.
pub use spec_spine_types as types;
pub use spec_spine_types::{Frontmatter, REGISTRY_SCHEMA_VERSION, Registry, SpecRecord, Violation};

pub use compile::{CompileOutcome, MAX_UNDECLARED_EXTRA_FRONTMATTER, compile};
pub use query::{
    ListFilter, RelationshipView, StatusReport, list, load_registry, relationships, show,
    status_report,
};

// ===== JSON-in / JSON-out facade (the FFI seam) =====

/// Compile the corpus under `repo_root`, returning the registry as JSON.
///
/// `config_json` is a JSON object matching [`Config`] (`"{}"` ⇒ defaults). The
/// returned string is the canonical `registry.json`; the caller inspects its
/// embedded `validation.passed`.
pub fn compile_json(config_json: &str, repo_root: &str) -> Result<String, Error> {
    let config = config_from_json(config_json)?;
    let outcome = compile(&config, std::path::Path::new(repo_root))?;
    Ok(outcome.json)
}

/// Run a read-only query described by `request_json`.
///
/// Request shape: `{ "registry": "<registry.json text>", "op": "list" |
/// "show" | "status-report" | "relationships", "id"?: string, "status"?: string }`.
pub fn query_json(request_json: &str) -> Result<String, Error> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Request {
        registry: String,
        op: Op,
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        status: Option<Status>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "kebab-case")]
    enum Op {
        List,
        Show,
        StatusReport,
        Relationships,
    }

    let request: Request = serde_json::from_str(request_json)
        .map_err(|e| Error::Parse(format!("invalid query request: {e}")))?;
    let registry = load_registry(request.registry.as_bytes())?;

    let json = match request.op {
        Op::List => {
            let filter = ListFilter {
                status: request.status,
            };
            to_json(&list(&registry, &filter))?
        }
        Op::Show => {
            let id = request
                .id
                .ok_or_else(|| Error::NotFound("missing 'id' for show".into()))?;
            to_json(show(&registry, &id)?)?
        }
        Op::StatusReport => to_json(&status_report(&registry))?,
        Op::Relationships => {
            let id = request
                .id
                .ok_or_else(|| Error::NotFound("missing 'id' for relationships".into()))?;
            to_json(&relationships(&registry, &id)?)?
        }
    };
    Ok(json)
}

/// Parse a `spec-spine.toml` and return the normalized [`Config`] as JSON.
pub fn load_config_json(toml_src: &str) -> Result<String, Error> {
    let config = load_config(toml_src)?;
    to_json(&config)
}

// --- facade helpers ---

fn config_from_json(config_json: &str) -> Result<Config, Error> {
    serde_json::from_str(config_json)
        .map_err(|e| Error::Config(format!("invalid config JSON: {e}")))
}

fn to_json<T: serde::Serialize>(value: &T) -> Result<String, Error> {
    serde_json::to_string(value).map_err(|e| Error::Schema(e.to_string()))
}
