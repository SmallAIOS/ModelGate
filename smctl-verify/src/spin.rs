//! SPIN/Promela protocol-verifier shell-out wrapper.
//!
//! Runs `spin -a` on each `[verify.protocol].specs`. v1 is exit-code
//! level only; counter-example rendering belongs in
//! `spin-protocol-runner-v1`.

use crate::shell::{Shell, discover_binary, run_against_sources};
use crate::{DiscoveryResult, Verifier, VerifyContext, VerifyReport};

const SHELL: Shell<'static> = Shell {
    name: "protocol",
    binary: "spin",
    version_args: &["-V"],
    run_args: &["-a"],
    install_hint: "install via `brew install spin` (macOS) or your distro's package manager",
    env_override: None,
};

#[derive(Debug, Default)]
pub struct SpinVerifier;

impl SpinVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl Verifier for SpinVerifier {
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
