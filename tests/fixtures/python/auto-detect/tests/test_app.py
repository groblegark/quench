"""Tests for app module."""

from src.app import add


def test_add() -> None:
    """Test add function."""
    assert add(1, 2) == 3
