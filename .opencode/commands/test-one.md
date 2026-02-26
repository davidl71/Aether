Run a single test by name pattern. Replace $TEST_NAME with the test to run (e.g., test_risk_calculator).

RUN ctest --test-dir build -R $TEST_NAME --output-on-failure -V
