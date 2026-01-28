"""Module with debugger statement - should fail."""


def process(data: str) -> str:
    """Process data with accidental debugger."""
    breakpoint()
    return data.upper()
