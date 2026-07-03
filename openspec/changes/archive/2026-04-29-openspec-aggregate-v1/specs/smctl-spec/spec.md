# smctl-spec Specification (delta)

## ADDED Requirements

### Requirement: Per-repo aggregation as the canonical model

`smctl-spec` SHALL treat each registered repo's `<repo>/openspec/` directory as the canonical home of that repo's specs. Aggregating commands (`list`) MUST walk every repo and concatenate; single-spec commands MUST resolve a name across every repo. The crate MUST NOT assume a single workspace-level openspec directory.

#### Scenario: List enumerates every repo

- **WHEN** `list_specs_across` is invoked with two repo entries, each containing one active spec
- **THEN** the result MUST contain two entries, one per repo
- **AND** each entry MUST carry the repo name as a separate field

### Requirement: Aggregating list API

`smctl-spec` SHALL expose `list_specs_across(repos: &[(String, PathBuf)]) -> Vec<RepoSpecInfo>` where each `RepoSpecInfo` carries the repo name plus the existing `SpecInfo` payload (name, phase, task progress, validation flags). The slice MAY be empty (zero-repo workspace) and the result MUST then be empty without erroring.

#### Scenario: Empty workspace returns empty list

- **WHEN** `list_specs_across(&[])` is invoked
- **THEN** the result MUST be `Ok(vec![])`

### Requirement: Spec name resolution across repos

`smctl-spec` SHALL expose `find_spec_in_repos(repos, name) -> Result<RepoSpecRef, ResolveError>` that resolves a spec name against the registered repos with these rules:

1. A name containing `:` MUST be split into `(repo, name)` and looked up directly.
2. A bare name found in exactly one repo MUST resolve unambiguously.
3. A bare name found in multiple repos MUST return `ResolveError::Ambiguous { matches: Vec<RepoSpecRef> }`.
4. A name found in no repo MUST return `ResolveError::NotFound { name }`.

#### Scenario: Qualified name resolves directly

- **WHEN** the operator invokes `find_spec_in_repos(repos, "ModelGate:foo-v1")`
- **THEN** the result MUST be the spec ref with repo=ModelGate and name=foo-v1

#### Scenario: Bare name unambiguous

- **WHEN** only one registered repo declares spec `foo-v1`
- **THEN** `find_spec_in_repos(repos, "foo-v1")` MUST return that ref

#### Scenario: Bare name ambiguous

- **WHEN** two registered repos each declare spec `foo-v1`
- **THEN** `find_spec_in_repos(repos, "foo-v1")` MUST return `Ambiguous` with both matches in the payload

#### Scenario: Bare name not found

- **WHEN** no registered repo declares spec `nonexistent`
- **THEN** the result MUST be `NotFound { name: "nonexistent" }`

### Requirement: Per-repo archive

`smctl-spec` SHALL expose `archive_in_repo(openspec_dir, name) -> PathBuf` that moves the spec from `<openspec_dir>/changes/<name>/` to `<openspec_dir>/changes/archive/<YYYY-MM-DD>-<name>/`. The function MUST refuse to operate on a path that does not start with the given `openspec_dir`.

#### Scenario: Archive moves spec into the repo's archive dir

- **WHEN** `archive_in_repo("/repos/A/openspec", "foo-v1")` is invoked against an existing spec
- **THEN** the spec dir MUST be moved under `/repos/A/openspec/changes/archive/<date>-foo-v1/`

### Requirement: Synthetic workspace-root entry

When the workspace root contains an `openspec/` directory and the manifest does not register a repo whose path covers that directory, `smctl-spec` SHALL inject a synthetic repo entry named `_workspace` so legacy single-repo workspaces continue to work without configuration.

#### Scenario: Workspace-level openspec preserved

- **WHEN** `list_specs_across` is given an empty repo slice and the workspace root has its own `openspec/` populated with one active spec
- **THEN** the result MUST contain that spec under repo name `_workspace`

#### Scenario: De-duplication when an explicit repo covers the same path

- **WHEN** `list_specs_across` is given a repo entry whose absolute path matches the workspace root
- **THEN** the synthetic `_workspace` entry MUST NOT be added
