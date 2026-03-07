{
  "action": "parallelization",
  "duration_weight": 0.3,
  "method": "native_go",
  "output_path": "/home/david/Projects/trading/ib_box_spread_full_universal/docs/TASK_ANALYSIS_PARALLELIZATION.md",
  "parallel_groups": [
    {
      "priority": "high",
      "reason": "4 tasks at dependency level 0 can run in parallel",
      "tasks": [
        "T-1772887500608268454",
        "T-1772887222103963807",
        "T-1772887222158664215",
        "T-1772887222213114929"
      ]
    },
    {
      "priority": "medium",
      "reason": "4 tasks at dependency level 0 can run in parallel",
      "tasks": [
        "T-1772887500742229784",
        "T-1772887500807853882",
        "T-1772887222270264987",
        "T-1772887222348905245"
      ]
    },
    {
      "priority": "medium",
      "reason": "3 tasks at dependency level 2 can run in parallel",
      "tasks": [
        "T-1772609725545817000",
        "T-1772609701509971000",
        "T-1772609720556934000"
      ]
    },
    {
      "priority": "medium",
      "reason": "2 tasks at dependency level 1 can run in parallel",
      "tasks": [
        "T-1772609719082616000",
        "T-1772609681056059000"
      ]
    },
    {
      "priority": "medium",
      "reason": "2 tasks at dependency level 3 can run in parallel",
      "tasks": [
        "T-1772609710882785000",
        "T-1772609727587961000"
      ]
    },
    {
      "priority": "low",
      "reason": "7 tasks at dependency level 0 can run in parallel",
      "tasks": [
        "T-1772111114876005000",
        "T-1772887762798843917",
        "T-1772887222509770969",
        "T-1772887222569465548",
        "T-1772887222624798220",
        "T-1772887222913841962",
        "T-1772887222970694620"
      ]
    }
  ],
  "recommendations": [
    {
      "count": 22,
      "groups": 6,
      "message": "22 tasks can be executed in parallel across 6 groups",
      "type": "parallel_execution"
    }
  ],
  "report": "Parallelization Analysis\n========================\n\nTotal Tasks: 371\n\nParallel Execution Groups:\n\nGroup 1 (high priority):\n  Reason: 4 tasks at dependency level 0 can run in parallel\n  Tasks:\n    - T-1772887500608268454\n    - T-1772887222103963807\n    - T-1772887222158664215\n    - T-1772887222213114929\n\nGroup 2 (medium priority):\n  Reason: 4 tasks at dependency level 0 can run in parallel\n  Tasks:\n    - T-1772887500742229784\n    - T-1772887500807853882\n    - T-1772887222270264987\n    - T-1772887222348905245\n\nGroup 3 (medium priority):\n  Reason: 3 tasks at dependency level 2 can run in parallel\n  Tasks:\n    - T-1772609725545817000\n    - T-1772609701509971000\n    - T-1772609720556934000\n\nGroup 4 (medium priority):\n  Reason: 2 tasks at dependency level 1 can run in parallel\n  Tasks:\n    - T-1772609719082616000\n    - T-1772609681056059000\n\nGroup 5 (medium priority):\n  Reason: 2 tasks at dependency level 3 can run in parallel\n  Tasks:\n    - T-1772609710882785000\n    - T-1772609727587961000\n\nGroup 6 (low priority):\n  Reason: 7 tasks at dependency level 0 can run in parallel\n  Tasks:\n    - T-1772111114876005000\n    - T-1772887762798843917\n    - T-1772887222509770969\n    - T-1772887222569465548\n    - T-1772887222624798220\n    - T-1772887222913841962\n    - T-1772887222970694620\n\n",
  "success": true,
  "total_tasks": 371
}
