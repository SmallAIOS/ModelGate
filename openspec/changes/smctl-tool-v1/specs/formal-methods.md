# smctl Formal Methods Tooling Specification

## Overview

smctl integrates formal methods across three verification domains. This spec analyzes which formal methods tools best serve each domain, evaluating TLA+, Lean 4, Cedar, Alloy, P, SPIN, and Rego/OPA.

## Tool Landscape

### Currently used in SmallAIOS

| Tool | Language % | Domain | SmallAIOS Usage |
|---|---|---|---|
| **TLA+** | 1.6% | Temporal/behavioral | Memory allocator, scheduler, syscall dispatch, security gate state machine, policy update protocol |
| **Lean 4** | 0.3% | Theorem proving | Biba integrity lattice, registry well-formedness, tensor invariant soundness, label composition |
| **SPIN/Promela** | 0.4% | Protocol verification | Additional model checking for concurrent protocols |
| **MISRA-Rust** | — | Coding standard | Safety-critical kernel code discipline |

### Candidates for smctl

| Tool | Verification Style | Code Gen | Best For | Learning Curve |
|---|---|---|---|---|
| **TLA+** | Exhaustive model checking | No | Distributed/concurrent algorithm correctness proofs | Moderate–Hard |
| **Cedar** | SMT-based policy analysis + Lean-verified | Rust SDK | Authorization policies (RBAC, ABAC, MAC) | Easy |
| **Alloy** | Bounded SAT-based checking | No | Structural/relational properties | Easier |
| **P** | Systematic testing (not exhaustive) | Yes (C, C#) | Async event-driven state machines | Moderate |
| **Lean 4** | Interactive theorem proving | No | Mathematical proofs of correctness | Hard |
| **SPIN/Promela** | Exhaustive model checking | Yes (C) | Protocol verification, concurrency | Moderate |
| **Rego/OPA** | Datalog-based policy evaluation | No | General-purpose policy (less formal guarantees) | Easy |

## Where Each Tool Fits in smctl

### TLA+ — Keep as primary for behavioral properties

**Use for:**
- Git flow state machine (branch lifecycle transitions)
- Cross-repo merge ordering (validate-then-execute atomicity)
- Workspace state machine (init → configured → synced)
- Policy update protocol verification (already done in formal-type-gate)
- Security gate state machine verification (already done in formal-type-gate)

**Why TLA+ here:**
- SmallAIOS already has deep TLA+ investment — consistency matters
- These are fundamentally concurrent/distributed coordination problems (multi-repo = distributed)
- Exhaustive model checking catches corner cases that testing misses
- Amazon AWS uses TLA+ for exactly this class of problem (DynamoDB, S3)

**Limitation:** TLA+ cannot generate executable Rust code, so specs and implementation diverge over time. Mitigated by `smctl build --verify` running TLC in CI.

### Cedar — Recommended for MAC/authorization policy (replaces Alloy for this role)

**Use for:**
- SmallAIOS MAC policy definition and enforcement
- SecurityLabel-based access control rules (classification + Biba integrity + message type)
- Model whitelist authorization (which models can run on which boundaries)
- Trust boundary crossing authorization (which data flows are permitted)
- Policy analysis: "can any inference request from Low integrity reach a High integrity kernel path?"

**Why Cedar is the strongest fit:**

1. **Purpose-built for authorization.** Cedar is specifically designed for access control policy — unlike Alloy (general-purpose structural analysis) or TLA+ (behavioral model checking). Cedar has native concepts for principals, actions, resources, and conditions that map directly to SmallAIOS's security model.

2. **Formally verified in Lean 4.** Cedar's evaluator, validator, and symbolic compiler are all modeled and proved correct in Lean 4 — **the same proof assistant SmallAIOS already uses**. This isn't a coincidence; it's an alignment of verification toolchains. Cedar's validator soundness proof is 4,686 lines of Lean 4. The formal model is verified against the Rust implementation via differential random testing (millions of inputs, both must agree).

3. **Native Rust SDK.** Cedar's production implementation is Rust (`cedar-policy` crate). smctl can directly link against it — no FFI, no subprocess invocation, no language boundary. This is the tightest possible integration.

4. **SMT-based policy analysis.** Cedar's symbolic compiler translates policies into SMT-LIB formulas checked by CVC5. This enables precise questions:
   - "Is there any request that policy A allows but policy B denies?" (policy equivalence)
   - "Can a Low-integrity message ever reach a High-integrity boundary?" (information flow)
   - "Does every model in the whitelist have a valid trust boundary path?" (completeness)
   - Average encoding + solving time: 75.1ms — fast enough for CI integration.

5. **CNCF Sandbox project.** Cedar is now under the Cloud Native Computing Foundation, ensuring long-term governance and community investment. Adopted by AWS, Cloudflare, MongoDB, StrongDM.

6. **28–80x faster than alternatives.** Cedar's evaluator is 28.7–35.2x faster than OpenFGA and 42.8–80.8x faster than Rego. For runtime enforcement in SmallAIOS's security gate, this matters.

**How Cedar maps to the formal-type-gate:**

| SmallAIOS Concept | Cedar Mapping |
|---|---|
| `SecurityLabel.classification` | Cedar `context.classification` attribute |
| `SecurityLabel.integrity` (Biba) | Cedar `context.integrity` attribute with ordered comparison |
| `SecurityLabel.message_type` | Cedar resource type |
| `BoundaryDefinition` | Cedar resource (e.g., `SmallAIOS::Boundary::"network-ingress"`) |
| `EnforcementMode` | Cedar policy effect (`permit` / `forbid`) |
| `ModelWhitelist` | Cedar policy set: `permit(principal, action == Action::"load", resource) when { resource.hash in context.whitelist }` |
| 5-layer verification pipeline | Cedar policy set with ordered evaluation |

**Example Cedar policy for SmallAIOS MAC:**
```cedar
// Biba no-write-up: Low integrity cannot write to High integrity boundaries
forbid(
    principal,
    action == Action::"cross_boundary",
    resource
) when {
    principal.integrity < resource.min_integrity
};

// Model whitelist: only approved models can load
permit(
    principal == SmallAIOS::Process::"onnx-runtime",
    action == Action::"load_model",
    resource
) when {
    resource.hash in context.approved_model_hashes
};

// Inference routing: Medium integrity messages can reach inference boundaries
permit(
    principal,
    action == Action::"route_inference",
    resource
) when {
    principal.integrity >= IntegrityLevel::"Medium"
    && resource in SmallAIOS::BoundaryGroup::"inference-endpoints"
};
```

**smctl integration:**
- `smctl gate policy write` — Author Cedar policies for SmallAIOS MAC
- `smctl gate policy analyze` — Run Cedar's SMT-based analyzer to check policy properties
- `smctl gate policy test <request>` — Evaluate a specific request against the policy
- `smctl gate policy diff <old> <new>` — Cedar policy diff with semantic comparison

**Cedar + Lean 4 verification chain:**
```
Cedar policy (human-authored)
    ↓ Cedar validator (Lean 4 proved sound)
Validated policy
    ↓ Cedar symbolic compiler (Lean 4 proved sound + complete)
SMT-LIB formulas
    ↓ CVC5 solver
Property verification results
    ↓ smctl gate policy verify
Pass/fail + counterexamples
```

### Alloy — Narrowed to structural analysis (non-policy)

With Cedar handling the policy domain, Alloy's role narrows to:

- **Trust boundary graph topology** — Which boundaries connect to which, reachability analysis, cycle detection
- **Message type registry structure** — Well-formedness of the 64-entry registry, no duplicate IDs
- **Workspace configuration structure** — Repo dependency graph validation (no cycles, all deps present)

Alloy remains useful for these graph/structure problems, but it is no longer the primary tool for MAC policy verification. Cedar is purpose-built for that.

**If budget is constrained**, Alloy can be deferred entirely — Cedar covers the most critical policy verification, and TLA+ covers the behavioral properties that Alloy cannot.

### P — Strong candidate for async protocol testing

**Use for:**
- ModelGate inference request routing (async event-driven state machine)
- `smctl gate` ↔ ModelGate communication protocol
- Failover testing: what happens when a ModelGate instance drops mid-inference?
- Network boundary validation: model network ingress as P state machines

**Why P here:**
- P is designed for exactly what ModelGate does: **async event-driven state machines communicating via messages**
- P was used to ship Windows USB 3.0 drivers — similar safety criticality
- Amazon S3 used P for strong consistency protocol reasoning
- P's **code generation** means the P model can produce test harnesses or even implementation scaffolding
- P models **fault injection** as first-class events — critical for testing ModelGate resilience
- P integrates into build pipelines (relevant for `smctl build --verify`)

**Trade-off:** P does systematic testing, not exhaustive model checking. It finds bugs but cannot *prove* absence of bugs. For smctl's git flow state machine, TLA+'s exhaustive checking is preferable. For ModelGate's complex async protocols, P's testing approach scales better.

**Code generation advantage:** P can compile specs to C or C# test harnesses. While it doesn't generate Rust directly, the test harnesses can drive Rust code through FFI or subprocess invocation. This partially bridges the spec-to-implementation gap that TLA+ cannot address.

### Lean 4 — Keep as-is for mathematical proofs

**Use for (unchanged from current SmallAIOS usage):**
- Biba integrity lattice properties
- Registry well-formedness and invariant determinism
- Tensor invariant soundness against ONNX spec
- Label composition correctness
- Cryptographic property proofs (ML-DSA-65, ML-KEM-768)

**Why Lean 4 stays:** These are mathematical facts that require *proof*, not testing or model checking. Lean 4 is the right tool.

### SPIN/Promela — Keep for protocol-level verification

**Use for (unchanged from current SmallAIOS usage):**
- Low-level concurrent protocol verification
- IPC message ordering guarantees
- Complement to TLA+ for specific protocol properties

## Recommended Multi-Tool Strategy

```
┌──────────────────────────────────────────────────────────────┐
│                     smctl verification                        │
│                                                              │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────────┐ │
│  │   TLA+      │  │    Cedar     │  │         P           │ │
│  │             │  │              │  │                     │ │
│  │ Behavioral  │  │ Authorization│  │  Async protocol     │ │
│  │ properties  │  │ policies     │  │  testing            │ │
│  │             │  │              │  │                     │ │
│  │ • git flow  │  │ • MAC rules  │  │ • ModelGate routing │ │
│  │ • merge     │  │ • Biba enf.  │  │ • fault injection   │ │
│  │   ordering  │  │ • model auth │  │ • async protocols   │ │
│  │ • workspace │  │ • boundary   │  │ • network ingress   │ │
│  │   states    │  │   crossing   │  │                     │ │
│  │ • policy    │  │ • SMT-based  │  │                     │ │
│  │   updates   │  │   analysis   │  │                     │ │
│  └──────┬──────┘  └──────┬───────┘  └──────────┬──────────┘ │
│         │                │                      │            │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌───────────┴─────────┐  │
│  │  Lean 4     │  │   SPIN      │  │    MISRA-Rust       │  │
│  │  Proofs     │  │  Protocols  │  │    Standards        │  │
│  │  (+ Cedar   │  │             │  │                     │  │
│  │   proofs)   │  │             │  │                     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                                                              │
│              ┌──────────────┐  (optional)                    │
│              │    Alloy     │                                │
│              │  Structural  │                                │
│              │  (graphs,    │                                │
│              │   registry)  │                                │
│              └──────────────┘                                │
└──────────────────────────────────────────────────────────────┘
```

**Key insight: Cedar and Lean 4 form a unified verification chain.** Cedar's evaluator and symbolic compiler are formally verified *in* Lean 4. SmallAIOS already uses Lean 4 for security proofs. This means the entire path from policy authoring through evaluation to property verification is covered by a single proof framework.

## smctl Commands for Multi-Tool Verification

```
smctl build --verify                    # Run all verification tools
smctl build --verify --tla              # TLA+ model checking only
smctl build --verify --cedar            # Cedar policy analysis only
smctl build --verify --p                # P systematic testing only
smctl build --verify --lean             # Lean 4 proof checking only
smctl build --verify --spin             # SPIN model checking only
smctl build --verify --alloy            # Alloy structural checking only (optional)

smctl gate policy verify                # Cedar (authorization) + TLA+ (behavior)
smctl gate policy analyze               # Cedar SMT-based property analysis
smctl gate policy test <request.json>   # Evaluate a specific request against policy
smctl gate policy verify --behavioral   # TLA+ policy update protocol only

smctl gate models verify <name>         # Lean 4 (tensor proofs) + Cedar (model auth policy)
smctl gate test <model> --fuzz          # P-based fault injection testing

smctl flow verify                       # TLA+ git flow state machine checking
```

## MCP Tools for Formal Verification

| MCP Tool | Underlying Tool | What It Checks |
|---|---|---|
| `smctl_build_verify` | All | Full verification suite |
| `smctl_gate_policy_verify` | Cedar + TLA+ | Policy authorization + behavioral properties |
| `smctl_gate_policy_analyze` | Cedar (SMT/CVC5) | Policy property analysis (equivalence, reachability, completeness) |
| `smctl_gate_policy_test` | Cedar evaluator | Single request evaluation against policy |
| `smctl_gate_models_verify` | Lean 4 + Cedar | Model tensor proofs + model authorization policy |
| `smctl_gate_test_fuzz` | P | Fault injection on async protocols |
| `smctl_flow_verify` | TLA+ | Git flow state machine correctness |

## Tool Comparison Matrix

| Concern | TLA+-Only | Cedar + TLA+ + P + Lean 4 |
|---|---|---|
| MAC policy definition | Hand-rolled in TLA+ (unnatural) | Cedar: purpose-built policy language |
| Policy formal verification | None (TLA+ verifies behavior, not policy) | Cedar: Lean 4-verified evaluator + SMT analysis |
| Rust integration | Shell out to TLC | Cedar: native Rust crate (`cedar-policy`) |
| Lean 4 alignment | Separate toolchains | Cedar is *verified in* Lean 4 — same proof chain |
| Async protocol testing | Manual trace extraction | P: code gen + fault injection |
| Developer accessibility | High barrier | Cedar is easy to read/write; P is programming-like |
| Runtime enforcement | Must re-implement in Rust | Cedar evaluator runs in-process at 28–80x faster than Rego |
| Exhaustive proof | TLA+ excels here | TLA+ still used where exhaustive proof needed |

## Why NOT Rego/OPA

Rego/OPA is the most widely deployed policy language, but has significant disadvantages for SmallAIOS:

| Concern | Rego/OPA | Cedar |
|---|---|---|
| **Formal verification** | No formal proofs of evaluator correctness | Evaluator proved correct in Lean 4 |
| **Policy analysis** | Testing-based only | SMT-based with soundness + completeness proofs |
| **Performance** | Baseline | 42–80x faster evaluation |
| **Determinism** | Can produce non-deterministic results | Guaranteed deterministic by design |
| **Type safety** | Dynamically typed | Static validation with proved-sound validator |
| **Rust SDK** | Go-native; Rust requires FFI/WASM | Native Rust crate |
| **Safety-critical suitability** | Not designed for it | Designed for verifiable authorization |

Rego's expressiveness is a liability in a safety-critical kernel context — it allows constructs that Cedar deliberately excludes to maintain formal verifiability.

## Why NOT Alloy as primary policy tool

Alloy is excellent for structural analysis but is **not a policy language**:

- No authorization concepts (permit/forbid, principal/action/resource)
- No runtime evaluation — analysis only
- No Rust SDK
- No Lean 4 proof chain
- Would require translating Cedar-like concepts into relational logic

Alloy remains useful for graph topology analysis (trust boundary connectivity, dependency cycles) but should not be the primary MAC policy tool.

## Disadvantages / Costs of Multi-Tool

| Cost | Mitigation |
|---|---|
| **Multiple tool installations** | `smctl` bundles or manages tool dependencies; `smctl setup --verify` installs all tools |
| **Multiple specification languages** | Each tool in its sweet spot; Cedar and TLA+ cover 80% of needs |
| **Team learning curve** | Cedar is easier than TLA+; P is programming-like; incremental adoption |
| **Spec synchronization** | `smctl spec validate` checks that all relevant artifacts exist and are consistent |
| **CI complexity** | `smctl build --verify` abstracts tool invocation; CI just runs one command |
| **Cedar is newer** | CNCF Sandbox, backed by AWS, adopted by Cloudflare/MongoDB; maturity growing fast |

## Recommendation

**Primary stack: Cedar + TLA+ + P + Lean 4 + SPIN + MISRA-Rust.**

| Tool | Role | Priority |
|---|---|---|
| **Cedar** | MAC policy definition, authorization, SMT analysis | **v0.1** — core to security gate |
| **TLA+** | Behavioral properties (git flow, policy updates, merge ordering) | **v0.1** — already in use |
| **Lean 4** | Mathematical proofs (integrity lattice, tensor invariants, Cedar proofs) | **v0.1** — already in use |
| **P** | Async protocol testing (ModelGate routing, fault injection) | **v0.2** — after core gate works |
| **SPIN** | Protocol-level concurrent verification | **v0.1** — already in use |
| **Alloy** | Structural analysis (optional, graph topology) | **v0.3** — nice-to-have |
| **MISRA-Rust** | Coding standards | **v0.1** — already in use |

The key insight: **Cedar fills the gap that Alloy, TLA+, and Rego each partially address.** It's a purpose-built authorization policy language with formal verification guarantees backed by the same Lean 4 proof system SmallAIOS already uses, with a native Rust SDK. For a security-critical unikernel with MAC enforcement, this is the right tool.

## References

- [Cedar language](https://github.com/cedar-policy) — CNCF Sandbox, Apache 2.0
- [Cedar formal spec in Lean 4](https://github.com/cedar-policy/cedar-spec) — Proofs of evaluator, validator, and symbolic compiler
- [Cedar paper (OOPSLA 2024)](https://www.amazon.science/publications/cedar-a-new-language-for-expressive-fast-safe-and-analyzable-authorization)
- [Lean Powers Cedar at AWS](https://lean-lang.org/use-cases/cedar/)
- [P language](https://github.com/p-org/P) — Microsoft Research
- [TLA+ at AWS](https://lamport.azurewebsites.net/tla/formal-methods-amazon.pdf)
- [Alloy](https://alloytools.org/)
- [NIST SP 800-192: Access Control Policy Verification](https://nvlpubs.nist.gov/nistpubs/specialpublications/nist.sp.800-192.pdf)
