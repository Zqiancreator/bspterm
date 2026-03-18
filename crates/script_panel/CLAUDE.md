# script_panel

Panel for Python script discovery, execution, and output display. Integrates with terminal_scripting for automation.

## Module Structure

```
src/
├── script_panel.rs        # Main panel UI and initialization
├── script_runner.rs       # Cross-platform Python execution
├── script_params.rs       # Parameter declaration parsing
└── script_params_modal.rs # Parameter input UI modal
```

## Key Types

| Type | Purpose |
|------|---------|
| `ScriptPanel` | Main panel (Panel, Focusable, Render) |
| `ScriptRunner` | Cross-platform Python executor |
| `ScriptStatus` | NotStarted/Running/Finished(i32)/Failed(String) |
| `ScriptEntry` | Script metadata (name, path) |
| `ScriptParams` | Parsed parameter declarations from script |
| `ScriptParam` | Single parameter definition (name, type, default, etc.) |
| `ParamType` | String/Number/Boolean/Choice/Password |
| `ScriptParamsModal` | Modal dialog for parameter input |

## Dependencies

- `python_runtime` - Python interpreter discovery (bundled → system fallback)
- `terminal_scripting` - ScriptingServer, terminal registry
- `workspace` - Panel framework, ModalView
- `gpui` - UI primitives
- `ui` - Shared UI components
- `editor` - Text input fields for parameters

## Script Parameters

Scripts can declare parameters using `@params...@end_params` blocks in their docstring:

```python
"""
@params
- host: string
  description: Target IP
  required: true
  default: "192.168.1.1"

- port: number
  default: 22

- protocol: choice
  choices: ["SSH", "Telnet"]
@end_params
"""

from bspterm import params
host = params.host
```

When a script with parameters is run:
1. `ScriptParams::parse_from_script()` extracts the @params block
2. `ScriptParamsModal` displays input fields for each parameter
3. User fills in values and clicks "Run Script"
4. Parameters are passed as `BSPTERM_PARAM_*` environment variables

**Example Scripts:**
- `assets/scripts/disp_boa.py` - Simple example with one required string parameter
- `assets/scripts/example_with_params.py` - Comprehensive example with all parameter types

## Common Tasks

**Add a default script:**
1. Add script to `assets/scripts/`
2. Update `script/bundle-default-config` (Linux) and `script/bundle-default-config.ps1` (Windows) to include the script in default config zip

**Add script execution option:**
1. Update `ScriptRunner::start()` in `script_runner.rs`
2. Pass new environment variables if needed

**Add new parameter type:**
1. Add variant to `ParamType` enum in `script_params.rs`
2. Update `parse_param_type()` function
3. Add UI rendering in `render_param_input()` in `script_params_modal.rs`

## Testing

```sh
cargo test -p script_panel
cargo test -p script_panel script_params  # Parameter parsing tests
```

## Pitfalls

- Scripts directory: `~/.config/bspterm/scripts/`
- `bspterm.py` is auto-installed but excluded from script list
- Python executable resolved via `python_runtime::python_executable()` (bundled first, then system PATH)
- Environment variables passed to scripts:
  - `BSPTERM_SOCKET` - Unix socket connection string
  - `PYTHONPATH` - Includes scripts directory + user site-packages (`python_runtime::user_site_packages()`)
  - `BSPTERM_CURRENT_TERMINAL` - Focused terminal UUID
  - `BSPTERM_PARAM_*` - Script parameters (uppercase names)
- Cross-platform I/O:
  - Unix: Uses `fcntl()` for non-blocking I/O
  - Windows: Uses `PeekNamedPipe()` for non-blocking reads
  - Windows: `CREATE_NO_WINDOW` flag hides console
- Panel docks on Left by default (priority: 20), can be moved to Right but not Bottom
- Parameter modal uses `WeakEntity<ScriptPanel>` to call back after user input
