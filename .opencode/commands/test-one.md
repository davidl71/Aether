Run a single test by name pattern. Pass the test name as $ARGUMENTS (for example `test_risk_calculator`).

RUN ctest --test-dir build -R $ARGUMENTS --output-on-failure -V
