import pytest
import time
import fibonacci


def test_basic():
    """Test basic Fibonacci sequence values"""
    assert fibonacci.fib(0) == 0
    assert fibonacci.fib(1) == 1
    assert fibonacci.fib(2) == 1
    assert fibonacci.fib(3) == 2
    assert fibonacci.fib(4) == 3
    assert fibonacci.fib(5) == 5
    assert fibonacci.fib(6) == 8
    assert fibonacci.fib(10) == 55


def test_performance():
    """Ensure the implementation is efficient for large numbers"""
    start_time = time.time()
    result = fibonacci.fib(50)
    end_time = time.time()

    # Should compute fib(50) quickly (< 1 second with dynamic programming)
    assert end_time - start_time < 1.0
    assert result == 12586269025  # Known value for fib(50)


def test_edge_cases():
    """Test edge cases"""
    assert fibonacci.fib(0) == 0
    assert fibonacci.fib(1) == 1


if __name__ == "__main__":
    pytest.main([__file__, "-v"])