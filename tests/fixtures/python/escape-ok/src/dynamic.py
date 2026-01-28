"""Module with eval and proper comment - should pass."""


def calculate(expr: str) -> int:
    """Calculate expression dynamically."""
    # EVAL: User-provided math expression for calculator feature
    result = eval(expr)
    return result
