//! Lean 4 proof shell-out wrapper.
//!
//! Runs `lake build` at each `[verify.proof].roots`. v1 is exit-code
//! level only; diagnostic mapping belongs in `lean-proof-runner-v1`.

use crate::shell::{Shell, discover_binary, run_against_sources};
use crate::{DiscoveryResult, Verifier, VerifyContext, VerifyReport};

const SHELL: Shell<'static> = Shell {
    name: "proof",
    binary: "lake",
    version_args: &["--version"],
    run_args: &["build"],
    install_hint: "install Lean 4 via elan: https://leanprover.github.io/lean4/doc/setup.html",
    env_override: None,
};

#[derive(Debug, Default)]
pub struct LeanVerifier;

impl LeanVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl Verifier for LeanVerifier {
    fn name(&self) -> &'static str {
        SHELL.name
    }
    fn discover(&self) -> DiscoveryResult {
        discover_binary(&SHELL)
    }
    fn run(&self, ctx: &VerifyContext) -> VerifyReport {
        run_against_sources(&SHELL, ctx)
    }
}
