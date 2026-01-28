# Python Language Adapter

Python-specific behavior for quench checks.

## Detection

Python adapter activates when any of these files exist:
- `pyproject.toml`
- `setup.py`
- `setup.cfg`
- `requirements.txt` (fallback)

## Profile Defaults

When using [`quench init --with python`](../01-cli.md#explicit-profiles), the following opinionated defaults are configured:

```toml
[python]
# Patterns can be overridden
# source = ["**/*.py"]
# tests = ["tests/**/*.py", "test_*.py", "*_test.py", "conftest.py"]
# ignore = [".venv/", "__pycache__/", ".mypy_cache/", ".pytest_cache/", "dist/", "build/", "*.egg-info/"]

[python.suppress]
check = "comment"

[python.suppress.test]
check = "allow"

[python.policy]
lint_changes = "standalone"
lint_config = [
    "pyproject.toml",  # when [tool.ruff], [tool.black], etc. present
    "ruff.toml",
    ".ruff.toml",
    ".flake8",
    ".pylintrc",
    "pylintrc",
    "mypy.ini",
    ".mypy.ini",
    "setup.cfg",  # when [flake8] or [mypy] sections present
]

[[check.escapes.patterns]]
pattern = "eval("
action = "comment"
comment = "# EVAL:"
advice = "Add a # EVAL: comment explaining the use case."

[[check.escapes.patterns]]
pattern = "exec("
action = "comment"
comment = "# EXEC:"
advice = "Add a # EXEC: comment explaining the use case."

[[check.escapes.patterns]]
pattern = "__import__("
action = "comment"
comment = "# DYNAMIC:"
advice = "Add a # DYNAMIC: comment explaining why dynamic import is needed."

[[check.escapes.patterns]]
pattern = "breakpoint()"
action = "forbid"
advice = "Remove debugger statement before committing."

[[check.escapes.patterns]]
pattern = "pdb.set_trace()"
action = "forbid"
advice = "Remove debugger statement before committing."

[[check.escapes.patterns]]
pattern = "import pdb"
action = "forbid"
advice = "Remove debugger import before committing."
```

**Landing the Plane items** (added to agent files when combined with `claude` or `cursor` profile):
- `ruff check .` or `flake8`
- `ruff format --check .` or `black --check .`
- `mypy .`
- `pytest`

## Default Patterns

```toml
[python]
source = ["**/*.py"]
tests = ["tests/**/*.py", "test_*.py", "*_test.py", "conftest.py"]
ignore = [".venv/", "__pycache__/", ".mypy_cache/", ".pytest_cache/", "dist/", "build/", "*.egg-info/"]
```

When `[python].tests` is not configured, patterns fall back to `[project].tests`, then to these defaults. See [Pattern Resolution](../02-config.md#pattern-resolution).

## Package Detection

### From pyproject.toml

```toml
[project]
name = "my-package"
```

### From setup.py

```python
setup(name="my-package", ...)
```

### From Directory Structure

- **src-layout**: `src/package_name/__init__.py`
- **flat-layout**: `package_name/__init__.py`

## Default Escape Patterns

| Pattern | Action | Comment Required |
|---------|--------|------------------|
| `eval(` | comment | `# EVAL:` |
| `exec(` | comment | `# EXEC:` |
| `__import__(` | comment | `# DYNAMIC:` |
| `breakpoint()` | forbid | - |
| `pdb.set_trace()` | forbid | - |
| `import pdb` | forbid | - |

Quench does not forbid usage directly, and assumes you are already running a linter like ruff or flake8. Instead it ensures escapes and suppressions are commented.

## Suppress

Controls `# noqa`, `# type: ignore`, and other lint suppression comments.

| Setting | Behavior |
|---------|----------|
| `"forbid"` | Never allowed |
| `"comment"` | Requires justification comment |
| `"allow"` | Always allowed (default) |

### Suppress Patterns

| Pattern | Default Action |
|---------|----------------|
| `# noqa` | comment (when configured) |
| `# noqa: CODE` | comment (when configured) |
| `# type: ignore` | comment (when configured) |
| `# type: ignore[code]` | comment (when configured) |
| `# pylint: disable=` | comment (when configured) |

### Configuration

```toml
[python.suppress]
check = "comment"              # forbid | comment | allow

[python.suppress.source]
allow = ["noqa: E501"]         # no comment needed for line length
forbid = ["type: ignore"]      # never allowed without code

[python.suppress.test]
check = "allow"                # tests can suppress freely
```

## Policy

Enforce lint configuration hygiene.

```toml
[python.policy]
lint_changes = "standalone"    # lint config changes must be standalone PRs
lint_config = [                # files that trigger standalone requirement
  "pyproject.toml",
  "ruff.toml",
  ".ruff.toml",
  ".flake8",
  ".pylintrc",
  "pylintrc",
  "mypy.ini",
  ".mypy.ini",
  "setup.cfg",
]
```

When `lint_changes = "standalone"`, changing any `lint_config` files alongside source/test changes fails:

```
python: FAIL
  lint config changes must be standalone
    Changed: pyproject.toml
    Also changed: src/parser.py, src/lexer.py
  Submit lint config changes in a separate PR.
```

## Configuration

```toml
[python]
# Source/test patterns (defaults shown; falls back to [project].tests if not set)
# source = ["**/*.py"]
# tests = ["tests/**/*.py", "test_*.py", "*_test.py", "conftest.py"]
# ignore = [".venv/", "__pycache__/", ".mypy_cache/", ".pytest_cache/", "dist/", "build/", "*.egg-info/"]

[python.cloc]
check = "error"                  # error | warn | off
# advice = "..."                 # Custom advice for oversized Python files

[python.suppress]
check = "comment"

[python.suppress.test]
check = "allow"

[python.policy]
lint_changes = "standalone"
lint_config = ["pyproject.toml", "ruff.toml", ".flake8"]
```

Test suites and coverage thresholds are configured in `[check.tests]`.
