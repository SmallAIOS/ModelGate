# smctl-log Specification (delta)

## MODIFIED Requirements

### Requirement: MSGID range allocation

The MSGID catalog SHALL allocate ranges per producer crate. The allocation table MUST be:

- `SMCTL-0001..0099` — smctl core (workspace, spec, flow, build)
- `SMCTL-0200..0299` — smctl-mcp
- `SMCTL-0300..0399` — modelgate-web
- `SMCTL-0400..0499` — smctl-quality
- `SMCTL-0500..0599` — smctl-verify

#### Scenario: Web MSGID is in range

- **WHEN** the operator inspects `MsgId::WebServerStarted.code()`
- **THEN** the returned value MUST be `301`
- **AND** the value MUST satisfy `(300..=399).contains(&code)`

#### Scenario: Quality MSGID is in range

- **WHEN** the operator inspects `MsgId::QualityCheckStarted.code()`
- **THEN** the returned value MUST be `400`
- **AND** the value MUST satisfy `(400..=499).contains(&code)`

#### Scenario: Verify MSGID is in range

- **WHEN** the operator inspects `MsgId::VerifyStarted.code()`
- **THEN** the returned value MUST be `501`
- **AND** the value MUST satisfy `(500..=599).contains(&code)`
