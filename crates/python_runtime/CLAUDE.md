# python_runtime

Cross-platform Python interpreter discovery and management. Provides bundled CPython 3.11 with system PATH fallback.

## Public API

| Function | Returns | Purpose |
|----------|---------|---------|
| `bundled_python_executable()` | `Option<PathBuf>` | Bundled Python path if it exists |
| `python_executable()` | `Result<PathBuf>` | Bundled first, then system fallback |
| `user_site_packages()` | `PathBuf` | User site-packages directory |
| `ensure_user_site_packages()` | `Result<PathBuf>` | Creates site-packages dir if needed |

## Bundled Python Paths

| Platform | Path (relative to exe) |
|----------|----------------------|
| Linux | `<exe_dir>/../lib/python/bin/python3.11` |
| Windows | `<exe_dir>/../python/python.exe` |

## User Site-Packages Paths

| Platform | Path |
|----------|------|
| Linux | `~/.config/bspterm/python/site-packages/` |
| Windows | `%LOCALAPPDATA%/Bspterm/python/site-packages/` |

## System Fallback

When bundled Python is not found (e.g., dev builds), tries system PATH candidates in order: `python3` → `python` → `py`. Each candidate is validated by running `python -c "print(1 + 2)"` and checking output is `"3"`.

## Consumers

- **script_panel** (`script_runner.rs`) — uses `python_executable()` to run automation scripts, adds `user_site_packages()` to `PYTHONPATH`
- **cli** (`main.rs`) — `bspterm --python <args>` forwards to `python_executable()` with `PYTHONPATH` set to user site-packages

## Bundle Scripts

- `script/bundle-python` (Linux) — downloads CPython 3.11 from python-build-standalone, strips test/idle/tkinter
- `script/bundle-python.ps1` (Windows) — same for Windows
- Output: `target/python-dist/` — consumed by `script/bundle-linux` and CI workflow
