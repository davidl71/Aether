Generate Catch2 tests for the specified source file.

READ $ARGUMENTS
READ native/tests/test_risk_calculator.cpp

Write Catch2 v3 unit tests for the source file above. Follow the conventions from the example test file:
- File name: native/tests/test_<module>.cpp
- Use TEST_CASE with [module_name] tags and SECTION for sub-scenarios
- Test edge cases: zero, negative, boundary values
- Test trading-specific cases: invalid strikes, expired contracts, zero notional
- Use Catch::Matchers::WithinAbs for floating-point comparisons
- Do NOT require network or TWS connection
