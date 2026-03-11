List current Todo2 tasks via the local exarp-go CLI wrapper.

If $ARGUMENTS is provided, use it as the task status filter (for example `Todo` or `In Progress`).

RUN ./scripts/run_exarp_go.sh -tool task_workflow -args '{"action":"sync","sub_action":"list"}' -quiet

If $ARGUMENTS is present and the generic list above is not enough, run the same command again with a JSON status filter for $ARGUMENTS.

Display results as a readable table with ID, Priority, Status, and Name.
