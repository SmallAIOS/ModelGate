//! TLA+ model-checker shell-out wrapper.
//!
//! Runs `tlc` against `[verify.model].specs`. v1 is exit-code level
//! only; counter-example rendering belongs in `tla-plus-runner-v1`.

use crate::shell::{Shell, discover_binary, run_against_sources};
use crate::{DiscoveryResult, Verifier, VerifyContext, VerifyReport};

const SHELL: Shell<'static> = Shell {
    name: "model",
    binary: "tlc",
    version_args: &["-h"],
    run_args: &[],
    install_hint: "install the TLA+ Toolbox or run `pip install tlaplus-cli`",
};

#[derive(Debug, Default)]
pub struct TlaVerifier;

impl TlaVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl Verifier for TlaVerifier {
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
