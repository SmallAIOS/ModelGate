//! Verifier registry — owns one boxed [`Verifier`](super::Verifier)
//! per supported tool and dispatches per-name lookups for the CLI.
//!
//! The registry is intentionally minimal: an ordered `Vec` of boxed
//! verifiers. Membership is decided at registration time so the
//! ordering reflects the operator-facing subcommand list (`policy`,
//! `model`, `proof`, `protocol`).

use crate::Verifier;

/// Registry of available verifiers.
pub struct Registry {
    verifiers: Vec<Box<dyn Verifier>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    /// Construct an empty registry. Most callers want
    /// [`Registry::with_default_verifiers`] instead, which assembles
    /// the canonical set of every verifier shipped in this crate.
    pub fn new() -> Self {
        Self {
            verifiers: Vec::new(),
        }
    }

    /// Construct a registry preloaded with every shipped verifier in
    /// the canonical CLI order: policy (Cedar), model (TLA+), proof
    /// (Lean 4), protocol (SPIN/Promela).
    pub fn with_default_verifiers() -> Self {
        let mut r = Self::new();
        r.register(crate::CedarVerifier::new());
        r.register(crate::TlaVerifier::new());
        r.register(crate::LeanVerifier::new());
        r.register(crate::SpinVerifier::new());
        r
    }

    /// Register one verifier. The first registration of a given name
    /// wins; later duplicate-name registrations are dropped.
    pub fn register<V: Verifier + 'static>(&mut self, verifier: V) {
        if self
            .verifiers
            .iter()
            .any(|existing| existing.name() == verifier.name())
        {
            return;
        }
        self.verifiers.push(Box::new(verifier));
    }

    /// Look up a verifier by stable name. Returns `None` when the name
    /// isn't registered.
    pub fn find(&self, name: &str) -> Option<&dyn Verifier> {
        self.verifiers
            .iter()
            .find(|v| v.name() == name)
            .map(|b| b.as_ref())
    }

    /// Iterate verifiers in registration order.
    pub fn iter(&self) -> impl Iterator<Item = &dyn Verifier> {
        self.verifiers.iter().map(|b| b.as_ref())
    }

    /// Number of registered verifiers.
    pub fn len(&self) -> usize {
        self.verifiers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.verifiers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DiscoveryResult, VerifyContext, VerifyReport};

    /// Tiny fixture verifier — exists only to exercise the registry's
    /// dispatch shape. Real verifiers (Cedar, TLA+, …) ship in their
    /// own modules.
    struct StubVerifier {
        name: &'static str,
    }

    impl Verifier for StubVerifier {
        fn name(&self) -> &'static str {
            self.name
        }
        fn discover(&self) -> DiscoveryResult {
            DiscoveryResult::Found {
                path: "<stub>".into(),
                version: String::new(),
            }
        }
        fn run(&self, _ctx: &VerifyContext) -> VerifyReport {
            VerifyReport::empty(self.name, crate::Outcome::NoSources)
        }
    }

    #[test]
    fn register_and_find_round_trip() {
        let mut r = Registry::new();
        r.register(StubVerifier { name: "policy" });
        r.register(StubVerifier { name: "model" });
        assert_eq!(r.len(), 2);
        assert!(r.find("policy").is_some());
        assert!(r.find("model").is_some());
        assert!(r.find("missing").is_none());
    }

    #[test]
    fn duplicate_registration_is_dropped() {
        let mut r = Registry::new();
        r.register(StubVerifier { name: "policy" });
        r.register(StubVerifier { name: "policy" });
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn with_default_verifiers_includes_all_four() {
        let r = Registry::with_default_verifiers();
        assert_eq!(r.len(), 4);
        let names: Vec<_> = r.iter().map(|v| v.name()).collect();
        assert_eq!(names, vec!["policy", "model", "proof", "protocol"]);
    }

    #[test]
    fn iter_yields_in_registration_order() {
        let mut r = Registry::new();
        r.register(StubVerifier { name: "policy" });
        r.register(StubVerifier { name: "model" });
        r.register(StubVerifier { name: "proof" });
        let names: Vec<_> = r.iter().map(|v| v.name()).collect();
        assert_eq!(names, vec!["policy", "model", "proof"]);
    }
}
