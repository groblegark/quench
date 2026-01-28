"""Module with eval but no comment - should fail."""


def calculate(expr: str) -> int:
    """Calculate expression dynamically."""
    result = eval(expr)
    return result
