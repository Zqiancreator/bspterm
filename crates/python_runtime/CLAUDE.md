# python_runtime

Cross-platform Python interpreter discovery and management. Finds system Python via PATH.

## Public API

| Function | Returns | Purpose |
|----------|---------|---------|
| `python_executable()` | `Result<PathBuf>` | System Python from PATH (cached) |
| `user_site_packages()` | `PathBuf` | User site-packages directory |
| `ensure_user_site_packages()` | `Result<PathBuf>` | Creates site-packages dir if needed |

## System Python Discovery

Searches system PATH candidates in order: `python3` → `python` → `py`. Each candidate is validated by running `python -c "print(1 + 2)"` and checking output is `"3"`. On Windows, Microsoft Store App Execution Aliases are skipped.

## User Site-Packages Paths

| Platform | Path |
|----------|------|
| Linux | `~/.config/bspterm/python/site-packages/` |
| Windows | `%LOCALAPPDATA%/Bspterm/python/site-packages/` |

## Consumers

- **script_panel** (`script_runner.rs`) — uses `python_executable()` to run automation scripts, adds `user_site_packages()` to `PYTHONPATH`
- **cli** (`main.rs`) — `bspterm --python <args>` forwards to `python_executable()` with `PYTHONPATH` set to user site-packages
