# smctl Formal Methods Tooling Specification

## Overview

smctl integrates formal methods across three verification domains. This spec analyzes which formal methods tools best serve each domain, evaluating TLA+, Lean 4, Alloy, P, and SPIN — all of which have presence in the SmallAIOS ecosystem.

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
| **Alloy** | Bounded SAT-based checking | No | Structural/relational properties, access control | Easier |
| **P** | Systematic testing (not exhaustive) | Yes (C, C#) | Async event-driven state machines | Moderate |
| **Lean 4** | Interactive theorem proving | No | Mathematical proofs of correctness | Hard |
| **SPIN/Promela** | Exhaustive model checking | Yes (C) | Protocol verification, concurrency | Moderate |

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

### Alloy — Strong candidate for MAC policy structure

**Use for:**
- SecurityLabel lattice structure verification
- Trust boundary graph analysis (which boundaries connect to which, reachability)
- Model whitelist policy structure (is the whitelist well-formed?)
- Access control rule completeness (does every boundary have a covering rule?)

**Why Alloy here:**
- MAC policies are fundamentally **structural and relational** — "which principals can access which resources through which boundaries"
- Alloy's SAT-based bounded checking is fast for exploring policy configurations
- Alloy excels at **graph-like structures** with transitive closure — trust boundary graphs are exactly this
- Alloy's visual counterexample exploration helps developers understand *why* a policy is wrong
- Alloy is **easier to teach** than TLA+, lowering the barrier for security engineers to write policy specs

**Trade-off:** Alloy is a bounded model checker — it explores up to N instances. For SmallAIOS's fixed-size registry (64 message types, 32 whitelist entries), the bounds are small enough that Alloy's bounded checking is effectively exhaustive.

**Where Alloy would NOT help:** Temporal properties (ordering of policy updates, monotonic mode transitions). Those stay with TLA+.

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
┌─────────────────────────────────────────────────────────┐
│                    smctl verification                    │
│                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │   TLA+      │  │   Alloy     │  │       P         │ │
│  │             │  │             │  │                 │ │
│  │ Behavioral  │  │ Structural  │  │ Async protocol  │ │
│  │ properties  │  │ properties  │  │ testing         │ │
│  │             │  │             │  │                 │ │
│  │ • git flow  │  │ • MAC policy│  │ • ModelGate     │ │
│  │ • merge     │  │   structure │  │   routing       │ │
│  │   ordering  │  │ • boundary  │  │ • fault         │ │
│  │ • workspace │  │   graphs    │  │   injection     │ │
│  │   states    │  │ • whitelist │  │ • async         │ │
│  │ • policy    │  │   rules     │  │   protocols     │ │
│  │   updates   │  │             │  │                 │ │
│  └──────┬──────┘  └──────┬──────┘  └────────┬────────┘ │
│         │                │                   │          │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌────────┴────────┐ │
│  │  Lean 4     │  │   SPIN      │  │   MISRA-Rust    │ │
│  │  Proofs     │  │  Protocols  │  │   Standards     │ │
│  └─────────────┘  └─────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

## smctl Commands for Multi-Tool Verification

```
smctl build --verify                    # Run all verification tools
smctl build --verify --tla              # TLA+ model checking only
smctl build --verify --alloy            # Alloy structural checking only
smctl build --verify --p                # P systematic testing only
smctl build --verify --lean             # Lean 4 proof checking only
smctl build --verify --spin             # SPIN model checking only

smctl gate policy verify                # Alloy (structure) + TLA+ (behavior)
smctl gate policy verify --structural   # Alloy only
smctl gate policy verify --behavioral   # TLA+ only

smctl gate models verify <name>         # Lean 4 (tensor proofs) + Alloy (type structure)
smctl gate test <model> --fuzz          # P-based fault injection testing
```

## MCP Tools for Formal Verification

| MCP Tool | Underlying Tool | What It Checks |
|---|---|---|
| `smctl_build_verify` | All | Full verification suite |
| `smctl_gate_policy_verify` | Alloy + TLA+ | Policy structure + behavior |
| `smctl_gate_models_verify` | Lean 4 + Alloy | Model proofs + type structure |
| `smctl_gate_test_fuzz` | P | Fault injection on async protocols |
| `smctl_flow_verify` | TLA+ | Git flow state machine correctness |

## Advantages of Multi-Tool over TLA+-Only

| Concern | TLA+-Only | Multi-Tool |
|---|---|---|
| MAC policy structure | Awkward (TLA+ is behavioral, not structural) | Alloy: natural fit for relational access control |
| Async protocol testing | Manual test extraction from TLA+ traces | P: native code generation + fault injection |
| Developer accessibility | High barrier (mathematical notation) | Alloy is easier to learn; P is programming-like |
| Code generation | None | P generates test harnesses from specs |
| Counterexample exploration | Text-based traces | Alloy provides visual graph exploration |
| Exhaustive proof | TLA+ excels here | TLA+ still used where exhaustive proof needed |

## Disadvantages / Costs of Multi-Tool

| Cost | Mitigation |
|---|---|
| **Multiple tool installations** | `smctl` bundles or manages tool dependencies; `smctl setup --verify` installs all |
| **Multiple specification languages** | Each tool used for its strength; no tool covers >2 domains |
| **Team learning curve** | Alloy is easier than TLA+; P is programming-like; incremental adoption |
| **Spec synchronization** | `smctl spec validate` checks that all relevant artifacts exist and are consistent |
| **CI complexity** | `smctl build --verify` abstracts tool invocation; CI just runs one command |

## Recommendation

**Use all three new tools (Alloy + P alongside existing TLA+ + Lean 4 + SPIN), each in its sweet spot.** The SmallAIOS ecosystem is already multi-tool (TLA+ + Lean 4 + SPIN + MISRA-Rust). Adding Alloy for structural MAC policy verification and P for async ModelGate protocol testing fills genuine gaps without redundancy.

The key insight: **formal methods tools are not interchangeable.** Each is optimized for a specific problem shape. Using the wrong tool for a problem is worse than not verifying at all, because it creates false confidence.
