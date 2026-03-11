Run the project scorecard.

Use the local exarp-go CLI wrapper with `report` `action=overview`.

RUN ./scripts/run_exarp_go.sh -tool report -args '{"action":"overview"}' -json -quiet

Also gather key metrics:
- Count C++ source files and test files in native/
- Count Python, Rust, TypeScript files
- Show task completion stats

Present as a structured scorecard with scores and recommendations.
