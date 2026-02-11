# smctl MCP Server Specification

## Overview

`smctl serve --mcp` starts a Model Context Protocol (MCP) server that exposes all smctl capabilities as tools and resources. This enables AI coding assistants — Claude Code, Cursor, Windsurf, Cline, and any MCP-compatible client — to programmatically manage SmallAIOS workspaces, branches, specs, and builds.

## Architecture

```
┌──────────────────────────────────────┐
│  AI Coding Assistant                 │
│  (Claude Code / Cursor / Windsurf)   │
│                                      │
│  MCP Client                          │
│    │                                 │
│    │ JSON-RPC 2.0                    │
│    │ (stdio / SSE / streamable HTTP) │
└────┼─────────────────────────────────┘
     │
     ▼
┌──────────────────────────────────────┐
│  smctl serve --mcp                   │
│                                      │
│  ┌─────────┐  ┌──────────────────┐   │
│  │Transport│  │ Tool Registry    │   │
│  │ Layer   │──│                  │   │
│  │(stdio/  │  │ workspace_init   │   │
│  │ SSE/    │  │ workspace_status │   │
│  │ HTTP)   │  │ worktree_add     │   │
│  └─────────┘  │ flow_feature_*   │   │
│               │ spec_new         │   │
│  ┌─────────┐  │ build            │   │
│  │Resource │  │ gate_*           │   │
│  │Registry │  │ ...              │   │
│  └─────────┘  └──────────────────┘   │
│       │                │             │
│       ▼                ▼             │
│  ┌──────────────────────────┐        │
│  │  smctl Core Library      │        │
│  │  (same logic as CLI)     │        │
│  └──────────────────────────┘        │
└──────────────────────────────────────┘
```

## Transport Modes

### stdio (default)

```bash
smctl serve --mcp --stdio
```

The MCP server reads JSON-RPC messages from stdin and writes responses to stdout. This is the standard transport for local AI tools.

### SSE (Server-Sent Events)

```bash
smctl serve --mcp --sse --port 3100
```

Starts an HTTP server. The client connects via SSE for server-to-client messages and POST for client-to-server messages. Useful for web-based or remote AI assistants.

### Streamable HTTP

```bash
smctl serve --mcp --http --port 3100
```

Uses the newer MCP streamable HTTP transport where both directions use HTTP requests.

## Server Capabilities

The MCP server advertises the following capabilities during initialization:

```json
{
  "capabilities": {
    "tools": {},
    "resources": {
      "subscribe": true,
      "listChanged": true
    },
    "logging": {}
  },
  "serverInfo": {
    "name": "smctl",
    "version": "0.1.0"
  }
}
```

## MCP Tools

Each CLI subcommand maps to an MCP tool. Tools accept JSON parameters and return structured JSON results.

### Workspace Tools

#### `smctl_workspace_init`
```json
{
  "name": "smctl_workspace_init",
  "description": "Initialize a SmallAIOS multi-repo workspace",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {"type": "string", "description": "Workspace name"},
      "repos": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "url": {"type": "string"},
            "path": {"type": "string"}
          }
        }
      }
    },
    "required": ["name"]
  }
}
```

#### `smctl_workspace_status`
```json
{
  "name": "smctl_workspace_status",
  "description": "Show status of all repos in the workspace",
  "inputSchema": {
    "type": "object",
    "properties": {}
  }
}
```

### Worktree Tools

#### `smctl_worktree_add`
```json
{
  "name": "smctl_worktree_add",
  "description": "Create linked worktrees for parallel feature development",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {"type": "string", "description": "Worktree set name"},
      "repos": {
        "type": "array",
        "items": {"type": "string"},
        "description": "Repos to include (default: all)"
      },
      "branch": {"type": "string", "description": "Branch to check out (default: feature/<name>)"}
    },
    "required": ["name"]
  }
}
```

#### `smctl_worktree_list`
```json
{
  "name": "smctl_worktree_list",
  "description": "List active worktree sets with branch and status info",
  "inputSchema": {
    "type": "object",
    "properties": {}
  }
}
```

#### `smctl_worktree_remove`
```json
{
  "name": "smctl_worktree_remove",
  "description": "Remove a worktree set and optionally clean up branches",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {"type": "string"},
      "force": {"type": "boolean", "default": false}
    },
    "required": ["name"]
  }
}
```

### Git Flow Tools

#### `smctl_flow_feature_start`
```json
{
  "name": "smctl_flow_feature_start",
  "description": "Create a feature branch across repos, optionally with worktree",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {"type": "string", "description": "Feature name (without feature/ prefix)"},
      "repos": {"type": "array", "items": {"type": "string"}},
      "worktree": {"type": "boolean", "default": false}
    },
    "required": ["name"]
  }
}
```

#### `smctl_flow_feature_finish`
```json
{
  "name": "smctl_flow_feature_finish",
  "description": "Merge feature into develop across repos",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {"type": "string"},
      "squash": {"type": "boolean", "default": false},
      "no_delete": {"type": "boolean", "default": false}
    },
    "required": ["name"]
  }
}
```

#### `smctl_flow_release_start`
```json
{
  "name": "smctl_flow_release_start",
  "description": "Create a release branch from develop in all repos",
  "inputSchema": {
    "type": "object",
    "properties": {
      "version": {"type": "string", "description": "Semver version (e.g., 1.0.0)"}
    },
    "required": ["version"]
  }
}
```

#### `smctl_flow_release_finish`
```json
{
  "name": "smctl_flow_release_finish",
  "description": "Finalize release: merge to main+develop, tag, changelog",
  "inputSchema": {
    "type": "object",
    "properties": {
      "version": {"type": "string"}
    },
    "required": ["version"]
  }
}
```

#### `smctl_flow_hotfix_start` / `smctl_flow_hotfix_finish`

Same pattern as feature start/finish but sourcing from `main`.

### OpenSpec Tools

#### `smctl_spec_new`
```json
{
  "name": "smctl_spec_new",
  "description": "Create a new OpenSpec feature folder with scaffolded documents",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {"type": "string", "description": "Spec/feature name"}
    },
    "required": ["name"]
  }
}
```

#### `smctl_spec_status`
```json
{
  "name": "smctl_spec_status",
  "description": "Show spec progress: tasks done/total, open questions, linked branches",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {"type": "string", "description": "Spec name (omit for all specs)"}
    }
  }
}
```

#### `smctl_spec_validate`
```json
{
  "name": "smctl_spec_validate",
  "description": "Check spec completeness: required files exist, design has decisions, tasks defined",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {"type": "string"}
    },
    "required": ["name"]
  }
}
```

#### `smctl_spec_archive`
```json
{
  "name": "smctl_spec_archive",
  "description": "Archive a completed spec to openspec/changes/archive/",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {"type": "string"}
    },
    "required": ["name"]
  }
}
```

### Build Tools

#### `smctl_build`
```json
{
  "name": "smctl_build",
  "description": "Build repos in dependency order",
  "inputSchema": {
    "type": "object",
    "properties": {
      "repo": {"type": "string", "description": "Specific repo to build (default: all)"},
      "parallel": {"type": "boolean", "default": false},
      "test": {"type": "boolean", "default": false},
      "clean": {"type": "boolean", "default": false}
    }
  }
}
```

### ModelGate Tools

#### `smctl_gate_status`
```json
{
  "name": "smctl_gate_status",
  "description": "Show ModelGate instance health and status",
  "inputSchema": {"type": "object", "properties": {}}
}
```

#### `smctl_gate_models_list`
```json
{
  "name": "smctl_gate_models_list",
  "description": "List registered ONNX models",
  "inputSchema": {"type": "object", "properties": {}}
}
```

#### `smctl_gate_models_add`
```json
{
  "name": "smctl_gate_models_add",
  "description": "Register a new ONNX model with ModelGate",
  "inputSchema": {
    "type": "object",
    "properties": {
      "path": {"type": "string", "description": "Path to ONNX model file"}
    },
    "required": ["path"]
  }
}
```

#### `smctl_gate_test`
```json
{
  "name": "smctl_gate_test",
  "description": "Run test inference against a registered model",
  "inputSchema": {
    "type": "object",
    "properties": {
      "model": {"type": "string"},
      "input": {"type": "string", "description": "Path to input data file"}
    },
    "required": ["model", "input"]
  }
}
```

## MCP Resources

Resources provide read-only context that AI assistants can inspect.

| URI Pattern | Description | MIME Type |
|---|---|---|
| `smctl://workspace/config` | Current workspace.toml | `application/toml` |
| `smctl://workspace/status` | Repo statuses JSON | `application/json` |
| `smctl://worktree/list` | Active worktrees JSON | `application/json` |
| `smctl://flow/branches` | All flow branches | `application/json` |
| `smctl://spec/list` | All specs with status | `application/json` |
| `smctl://spec/{name}/proposal` | Proposal markdown | `text/markdown` |
| `smctl://spec/{name}/design` | Design markdown | `text/markdown` |
| `smctl://spec/{name}/tasks` | Tasks markdown | `text/markdown` |
| `smctl://spec/{name}/status` | Task completion JSON | `application/json` |
| `smctl://gate/models` | Registered models | `application/json` |
| `smctl://gate/routes` | Routing table | `application/json` |

Resources support subscription. When workspace state changes (branch switch, spec update, build completion), the server emits `notifications/resources/updated` so the AI assistant stays current.

## Client Configuration

### Claude Code

Add to project `.mcp.json`:
```json
{
  "mcpServers": {
    "smctl": {
      "command": "smctl",
      "args": ["serve", "--mcp", "--stdio"]
    }
  }
}
```

Or to `~/.claude/claude_desktop_config.json` for global availability.

### Cursor

Add to `.cursor/mcp.json`:
```json
{
  "mcpServers": {
    "smctl": {
      "command": "smctl",
      "args": ["serve", "--mcp", "--stdio"]
    }
  }
}
```

### Windsurf

Add to `~/.codeium/windsurf/mcp_config.json`:
```json
{
  "mcpServers": {
    "smctl": {
      "command": "smctl",
      "args": ["serve", "--mcp", "--stdio"]
    }
  }
}
```

### Remote / Web-based Assistants

For assistants that cannot use stdio, start the SSE server:
```bash
smctl serve --mcp --sse --port 3100
```

The client connects to `http://localhost:3100/sse` for the event stream and POSTs to `http://localhost:3100/messages` for requests.

## Security Considerations

- **Destructive operations:** Tools that delete branches, force-merge, or remove worktrees should include confirmation metadata. MCP clients that support user approval prompts will display these.
- **File system access:** The MCP server operates within the workspace directory. It does not access files outside the workspace root.
- **No credentials in responses:** Tool results must not include git credentials, tokens, or secrets.
- **Logging:** All MCP tool invocations are logged (at info level) for auditability. Use `SMCTL_LOG=debug` for full request/response logging.

## Error Handling

MCP tool errors follow JSON-RPC 2.0 error format:

```json
{
  "error": {
    "code": -32000,
    "message": "Feature branch 'foo' does not exist in repo SmallAIOS",
    "data": {
      "repo": "SmallAIOS",
      "branch": "feature/foo",
      "suggestion": "Run smctl_flow_feature_start first"
    }
  }
}
```

Error codes:
- `-32000` — General smctl error
- `-32001` — Workspace not initialized
- `-32002` — Git operation failed
- `-32003` — Spec validation failed
- `-32004` — Build failed
- `-32005` — ModelGate connection error
