//! smctl-verify — formal-verification surface for `smctl`.
//!
//! This crate backs the `smctl verify <verb>` command tree. Each
//! verifier (Cedar / TLA+ / Lean 4 / SPIN/Promela / discover) is an
//! implementation of the [`Verifier`] trait. The CLI dispatches into
//! a registry that owns one boxed implementation per supported tool.
//!
//! Status: scaffold only. The trait shape, registry, runner, and
//! Cedar end-to-end land in subsequent commits on this branch
//! (per `openspec/changes/formal-methods-v1/tasks.md`).

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles() {
        // Smoke test: the crate links and depends on cedar-policy
        // without conflicts. Real verifier tests land in the
        // per-module test files (cedar.rs, tla.rs, ...).
    }
}
