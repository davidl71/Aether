#!/usr/bin/env python3
"""
Simple test runner for security tests (fallback when pytest not available).
"""
import sys
import unittest
from pathlib import Path

# Add python directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

# Import test modules
from tests.test_security import (
    TestPathBoundaryEnforcer,
    TestRateLimiter,
    TestAccessControl,
    TestRateLimitMiddleware,
    TestRequireApiKey
)


def run_tests():
    """Run all security tests."""
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()
    
    # Add test classes (skip async tests for now - need asyncio support)
    suite.addTests(loader.loadTestsFromTestCase(TestPathBoundaryEnforcer))
    suite.addTests(loader.loadTestsFromTestCase(TestRateLimiter))
    suite.addTests(loader.loadTestsFromTestCase(TestAccessControl))
    # Note: RateLimitMiddleware and RequireApiKey tests require asyncio - skip for now
    
    # Run tests
    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)
    
    # Exit with appropriate code
    sys.exit(0 if result.wasSuccessful() else 1)


if __name__ == '__main__':
    run_tests()
