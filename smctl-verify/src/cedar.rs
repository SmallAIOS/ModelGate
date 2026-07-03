//! Cedar policy verifier.
//!
//! The headline `formal-methods-v1` integration: the only verifier in
//! v1 that runs end-to-end inside the `smctl` process. Every policy
//! file declared in `[verify.policy].sources` is parsed with the
//! `cedar-policy` crate; parse errors translate to three-part
//! remediation diagnostics.
//!
//! Schema-aware validation (`Validator::validate`) is in scope for a
//! later iteration once we settle on a per-repo schema discovery
//! convention. For v1 the verifier reports a policy file as `passed`
//! when it parses cleanly and the policy set is non-empty.

use std::path::PathBuf;

use cedar_policy::PolicySet;
use glob::glob;

use crate::{DiscoveryResult, Outcome, SourceRow, Verifier, VerifyContext, VerifyReport};

/// Cedar implementation of [`Verifier`].
#[derive(Debug, Default)]
pub struct CedarVerifier;

impl CedarVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl Verifier for CedarVerifier {
    fn name(&self) -> &'static str {
        "policy"
    }

    fn discover(&self) -> DiscoveryResult {
        // Cedar is a Rust dep — always reachable when this crate
        // compiles. The version is the major Cedar series we pinned.
        DiscoveryResult::Found {
            path: "<rust-dep cedar-policy>".into(),
            version: "4".into(),
        }
    }

    fn run(&self, ctx: &VerifyContext) -> VerifyReport {
        let mut report = VerifyReport::empty(self.name(), Outcome::NoSources);

        if ctx.manifest.sources.is_empty() {
            return report;
        }

        let mut any_failed = false;
        let mut any_processed = false;

        for (repo_name, repo_path) in &ctx.repos {
            for pattern in &ctx.manifest.sources {
                let absolute = repo_path.join(pattern);
                let glob_pattern = match absolute.to_str() {
                    Some(s) => s.to_string(),
                    None => {
                        report.diagnostics.push(format!(
                            "skipped non-utf8 glob pattern under repo {repo_name}: {pattern}"
                        ));
                        continue;
                    }
                };

                let entries = match glob(&glob_pattern) {
                    Ok(it) => it,
                    Err(e) => {
                        report.diagnostics.push(format!(
                            "invalid glob '{pattern}' under repo {repo_name}: {e}. Fix the pattern in [verify.policy].sources and re-run.",
                        ));
                        any_failed = true;
                        continue;
                    }
                };

                for entry in entries {
                    let path = match entry {
                        Ok(p) => p,
                        Err(e) => {
                            report.diagnostics.push(format!(
                                "could not read entry under repo {repo_name}: {e}. Check filesystem permissions on the matched path.",
                            ));
                            any_failed = true;
                            continue;
                        }
                    };
                    any_processed = true;
                    let row = check_policy_file(&path);
                    if matches!(row.outcome, Outcome::Failed) {
                        any_failed = true;
                        report
                            .diagnostics
                            .extend(row.note.lines().map(str::to_string));
                    }
                    report.sources.push(SourceRow::plain(
                        path.display().to_string(),
                        row.outcome,
                        row.note,
                    ));
                }
            }
        }

        report.outcome = match (any_processed, any_failed) {
            (false, _) => Outcome::NoSources,
            (true, true) => Outcome::Failed,
            (true, false) => Outcome::Passed,
        };
        report
    }
}

/// Per-file check. Reads the file, parses it as a Cedar policy set,
/// and produces a `SourceRow`-shaped result. The `note` field carries
/// either the policy count (on pass) or the three-part remediation
/// (on fail).
fn check_policy_file(path: &PathBuf) -> CheckResult {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            return CheckResult {
                outcome: Outcome::Failed,
                note: format!(
                    "could not read {}: {e}. The file may have moved or its permissions changed; verify it exists and is readable, then re-run `smctl verify policy`.",
                    path.display(),
                ),
            };
        }
    };
    let text = match std::str::from_utf8(&bytes) {
        Ok(s) => s,
        Err(e) => {
            return CheckResult {
                outcome: Outcome::Failed,
                note: format!(
                    "{} is not valid UTF-8 ({e}). Cedar policies must be UTF-8 encoded; re-export the file with the correct encoding.",
                    path.display(),
                ),
            };
        }
    };

    match PolicySet::from_str(text) {
        Ok(set) => {
            let policy_count = set.policies().count();
            if policy_count == 0 {
                CheckResult {
                    outcome: Outcome::Failed,
                    note: format!(
                        "{} parsed but contains no policies. Either add at least one `permit` or `forbid` statement, or remove the file from [verify.policy].sources.",
                        path.display(),
                    ),
                }
            } else {
                CheckResult {
                    outcome: Outcome::Passed,
                    note: format!("{policy_count} policy/policies"),
                }
            }
        }
        Err(errors) => CheckResult {
            outcome: Outcome::Failed,
            note: format!(
                "{} failed to parse: {errors}. Fix the syntax errors above and re-run `smctl verify policy`. See https://docs.cedarpolicy.com/policies/syntax-policy.html for the policy grammar.",
                path.display(),
            ),
        },
    }
}

struct CheckResult {
    outcome: Outcome,
    note: String,
}

// `PolicySet::from_str` requires the trait to be in scope. Importing
// at module top would shadow the std FromStr; bring it in narrowly.
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::io::Write;

    use crate::{VerifyContext, VerifyManifest};

    fn ctx_with_repo(repo_root: PathBuf, sources: Vec<String>) -> VerifyContext {
        let mut repos = BTreeMap::new();
        repos.insert("test-repo".to_string(), repo_root);
        VerifyContext {
            workspace_root: PathBuf::from("/tmp"),
            repos,
            manifest: VerifyManifest {
                sources,
                fail_on: "any".into(),
                ..VerifyManifest::default()
            },
            strict: false,
            verifier_filter: None,
        }
    }

    fn write(dir: &tempfile::TempDir, name: &str, body: &str) {
        let path = dir.path().join(name);
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(body.as_bytes()).unwrap();
    }

    #[test]
    fn discover_is_always_found() {
        let v = CedarVerifier::new();
        let d = v.discover();
        assert!(d.is_installed());
        match d {
            DiscoveryResult::Found { path, version } => {
                assert!(path.contains("cedar-policy"));
                assert_eq!(version, "4");
            }
            _ => panic!("expected Found"),
        }
    }

    #[test]
    fn no_sources_when_manifest_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = ctx_with_repo(dir.path().to_path_buf(), vec![]);
        let report = CedarVerifier::new().run(&ctx);
        assert_eq!(report.outcome, Outcome::NoSources);
        assert!(report.sources.is_empty());
    }

    #[test]
    fn well_formed_policy_passes() {
        let dir = tempfile::tempdir().unwrap();
        write(
            &dir,
            "allow.cedar",
            r#"permit(principal, action, resource);"#,
        );
        let ctx = ctx_with_repo(dir.path().to_path_buf(), vec!["*.cedar".into()]);
        let report = CedarVerifier::new().run(&ctx);
        assert_eq!(report.outcome, Outcome::Passed);
        assert_eq!(report.sources.len(), 1);
        assert_eq!(report.sources[0].outcome, Outcome::Passed);
        assert!(report.sources[0].note.contains("1 policy"));
    }

    #[test]
    fn malformed_policy_fails_with_remediation() {
        let dir = tempfile::tempdir().unwrap();
        // Missing closing paren.
        write(
            &dir,
            "broken.cedar",
            r#"permit(principal, action, resource;"#,
        );
        let ctx = ctx_with_repo(dir.path().to_path_buf(), vec!["*.cedar".into()]);
        let report = CedarVerifier::new().run(&ctx);
        assert_eq!(report.outcome, Outcome::Failed);
        assert_eq!(report.sources.len(), 1);
        assert_eq!(report.sources[0].outcome, Outcome::Failed);
        // Three-part remediation: file path + parse error + next-step command.
        assert!(report.sources[0].note.contains("broken.cedar"));
        assert!(
            report.sources[0].note.contains("smctl verify policy"),
            "note should suggest the next command, got: {}",
            report.sources[0].note
        );
    }

    #[test]
    fn empty_policy_set_fails() {
        let dir = tempfile::tempdir().unwrap();
        // Parses cleanly but no policies — operator error.
        write(&dir, "empty.cedar", "// just a comment\n");
        let ctx = ctx_with_repo(dir.path().to_path_buf(), vec!["*.cedar".into()]);
        let report = CedarVerifier::new().run(&ctx);
        assert_eq!(report.outcome, Outcome::Failed);
        assert!(report.sources[0].note.contains("contains no policies"));
    }

    #[test]
    fn complex_policy_with_conditions_passes() {
        let dir = tempfile::tempdir().unwrap();
        write(
            &dir,
            "rbac.cedar",
            r#"
            permit(
                principal == User::"alice",
                action == Action::"read",
                resource == Document::"public"
            );
            forbid(
                principal,
                action == Action::"write",
                resource
            ) when {
                resource has classification && resource.classification == "secret"
            };
            "#,
        );
        let ctx = ctx_with_repo(dir.path().to_path_buf(), vec!["*.cedar".into()]);
        let report = CedarVerifier::new().run(&ctx);
        assert_eq!(report.outcome, Outcome::Passed);
        assert!(report.sources[0].note.contains("2 polic"));
    }

    #[test]
    fn missing_files_glob_yields_no_sources() {
        let dir = tempfile::tempdir().unwrap();
        // Pattern matches nothing.
        let ctx = ctx_with_repo(
            dir.path().to_path_buf(),
            vec!["does-not-exist/*.cedar".into()],
        );
        let report = CedarVerifier::new().run(&ctx);
        assert_eq!(report.outcome, Outcome::NoSources);
    }
}
