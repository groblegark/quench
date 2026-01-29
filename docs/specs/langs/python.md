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
pattern = "breakpoint()"
action = "forbid"
in_tests = "forbid"
advice = "Remove breakpoint() before committing."

[[check.escapes.patterns]]
pattern = "pdb.set_trace()"
action = "forbid"
in_tests = "forbid"
advice = "Remove pdb.set_trace() before committing."

[[check.escapes.patterns]]
pattern = "import pdb"
action = "forbid"
in_tests = "forbid"
advice = "Remove import pdb before committing."

[[check.escapes.patterns]]
pattern = "from pdb import"
action = "forbid"
in_tests = "forbid"
advice = "Remove pdb import before committing."

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
pattern = "compile("
action = "comment"
comment = "# DYNAMIC:"
advice = "Add a # DYNAMIC: comment explaining why compile is necessary for code execution."
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

| Pattern | Action | Comment Required | In Tests |
|---------|--------|------------------|----------|
| `breakpoint()` | forbid | - | forbid |
| `pdb.set_trace()` | forbid | - | forbid |
| `import pdb` | forbid | - | forbid |
| `from pdb import` | forbid | - | forbid |
| `eval(` | comment | `# EVAL:` | allow |
| `exec(` | comment | `# EXEC:` | allow |
| `__import__(` | comment | `# DYNAMIC:` | allow |
| `compile(` | comment | `# DYNAMIC:` | allow |

**Debugger patterns** are forbidden even in test code to prevent accidental commits that break CI.

**Dynamic execution patterns** (eval, exec, __import__, compile) are allowed in tests without comments but require justification in source code.

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
