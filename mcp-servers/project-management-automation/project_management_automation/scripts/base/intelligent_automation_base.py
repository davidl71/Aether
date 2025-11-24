#!/usr/bin/env python3
"""
Intelligent Automation Base Class

This base class integrates:
- Tractatus Thinking: Understand WHAT to analyze (structure)
- Sequential Thinking: Plan HOW to analyze (workflow)
- Todo2: Track execution and create follow-up tasks
- NetworkX: Understand relationships and dependencies

All automation scripts should inherit from this class.
"""

import json
import logging
import os
import sys
from abc import ABC, abstractmethod
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Any

# Import MCP client (relative import for package)
try:
    from .mcp_client import get_mcp_client
    MCP_AVAILABLE = True
except ImportError:
    MCP_AVAILABLE = False
    logger.warning("MCP client not available, using fallback implementations")

# Project root detection - will be set by subclasses or tools
project_root = None

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class IntelligentAutomationBase(ABC):
    """Base class for intelligent automation scripts."""

    def __init__(self, config: Dict, automation_name: str, project_root: Optional[Path] = None):
        self.config = config
        self.automation_name = automation_name
        # Use provided project_root or try to detect it
        if project_root:
            self.project_root = Path(project_root)
        else:
            # Try to find project root by looking for .git, .todo2, or CMakeLists.txt
            current = Path(__file__).parent.parent.parent.parent
            while current != current.parent:
                if (current / '.git').exists() or (current / '.todo2').exists() or (current / 'CMakeLists.txt').exists():
                    self.project_root = current
                    break
                current = current.parent
            else:
                # Fallback to current working directory
                self.project_root = Path.cwd()

        # Initialize components
        self.tractatus_session = None
        self.sequential_session = None
        self.todo2_task = None
        self.networkx_graph = None

        # Results storage
        self.results = {
            'automation_name': automation_name,
            'timestamp': datetime.now().isoformat(),
            'workflow_steps': [],
            'findings': [],
            'recommendations': [],
            'followup_tasks': []
        }

    def run(self) -> Dict:
        """Main execution method - follows intelligent automation pattern."""
        logger.info(f"Starting intelligent automation: {self.automation_name}")

        try:
            # Step 1: Use Tractatus Thinking to understand structure
            self._tractatus_analysis()

            # Step 2: Use Sequential Thinking to plan workflow
            self._sequential_planning()

            # Step 3: Create Todo2 task for tracking
            self._create_todo2_task()

            # Step 4: Use NetworkX for dependency analysis (if applicable)
            self._networkx_analysis()

            # Step 5: Execute analysis (implemented by subclasses)
            analysis_results = self._execute_analysis()

            # Step 6: Generate insights using Tractatus
            insights = self._generate_insights(analysis_results)

            # Step 7: Store results in Todo2
            self._store_todo2_results(analysis_results, insights)

            # Step 8: Create follow-up tasks
            self._create_followup_tasks(analysis_results)

            # Step 9: Generate report
            report = self._generate_report(analysis_results, insights)

            # Step 10: Update Todo2 task
            self._update_todo2_complete()

            self.results['status'] = 'success'
            self.results['report'] = report

            logger.info(f"Intelligent automation completed: {self.automation_name}")
            return self.results

        except Exception as e:
            logger.error(f"Error in intelligent automation: {e}", exc_info=True)
            self.results['status'] = 'error'
            self.results['error'] = str(e)
            self._update_todo2_error(e)
            raise

    def _tractatus_analysis(self) -> None:
        """Use Tractatus Thinking to understand what to analyze."""
        logger.info("Starting Tractatus analysis...")

        concept = self._get_tractatus_concept()

        # Try to use Tractatus Thinking MCP server
        if MCP_AVAILABLE:
            try:
                mcp_client = get_mcp_client(self.project_root)
                result = mcp_client.call_tractatus_thinking('start', concept=concept)

                if result:
                    self.tractatus_session = {
                        'concept': concept,
                        'session_id': result.get('session_id'),
                        'components': result.get('components', []),
                        'dependencies': self._identify_dependencies(concept)
                    }
                    logger.info(f"Tractatus analysis complete: {len(self.tractatus_session['components'])} components identified")
                    return
            except Exception as e:
                logger.warning(f"Tractatus MCP call failed: {e}, using fallback")

        # Fallback to simplified analysis
        self.tractatus_session = {
            'concept': concept,
            'components': self._extract_components_from_concept(concept),
            'dependencies': self._identify_dependencies(concept)
        }
        logger.info(f"Tractatus analysis complete (fallback): {len(self.tractatus_session['components'])} components identified")

    def _sequential_planning(self) -> None:
        """Use Sequential Thinking to plan workflow."""
        logger.info("Starting Sequential planning...")

        problem = self._get_sequential_problem()

        # Try to use Sequential Thinking MCP server
        if MCP_AVAILABLE:
            try:
                mcp_client = get_mcp_client(self.project_root)
                result = mcp_client.call_sequential_thinking('start', problem=problem)

                if result:
                    self.sequential_session = {
                        'problem': problem,
                        'session_id': result.get('session_id'),
                        'steps': result.get('steps', []),
                        'current_step': 0
                    }
                    logger.info(f"Sequential planning complete: {len(self.sequential_session['steps'])} steps planned")
                    return
            except Exception as e:
                logger.warning(f"Sequential MCP call failed: {e}, using fallback")

        # Fallback to simplified planning
        self.sequential_session = {
            'problem': problem,
            'steps': self._plan_workflow_steps(problem),
            'current_step': 0
        }
        logger.info(f"Sequential planning complete (fallback): {len(self.sequential_session['steps'])} steps planned")

    def _create_todo2_task(self) -> None:
        """Create Todo2 task for tracking automation execution."""
        logger.info("Creating Todo2 task...")

        try:
            # Load Todo2 state
            todo2_path = self.project_root / '.todo2' / 'state.todo2.json'
            if todo2_path.exists():
                with open(todo2_path, 'r') as f:
                    todo2_data = json.load(f)

                # Create task entry
                task_id = f"AUTO-{datetime.now().strftime('%Y%m%d%H%M%S')}"
                task = {
                    'id': task_id,
                    'name': f"Automation: {self.automation_name}",
                    'content': f"Automated {self.automation_name} execution",
                    'status': 'in_progress',
                    'priority': 'medium',
                    'tags': ['automation', self.automation_name.lower().replace(' ', '-')],
                    'created': datetime.now().isoformat(),
                    'lastModified': datetime.now().isoformat(),
                    'dependencies': []
                }

                if 'todos' not in todo2_data:
                    todo2_data['todos'] = []

                todo2_data['todos'].append(task)

                # Save back
                with open(todo2_path, 'w') as f:
                    json.dump(todo2_data, f, indent=2)

                self.todo2_task = task
                logger.info(f"Todo2 task created: {task_id}")
            else:
                logger.warning("Todo2 state file not found, skipping task creation")
        except Exception as e:
            logger.warning(f"Failed to create Todo2 task: {e}")

    def _networkx_analysis(self) -> None:
        """Use NetworkX for dependency analysis if applicable."""
        if not self._needs_networkx():
            return

        logger.info("Starting NetworkX analysis...")

        try:
            import networkx as nx
            self.networkx_graph = self._build_networkx_graph()

            if self.networkx_graph and len(self.networkx_graph.nodes()) > 0:
                # Analyze graph
                analysis = {
                    'nodes': len(self.networkx_graph.nodes()),
                    'edges': len(self.networkx_graph.edges()),
                    'critical_path': self._find_critical_path(),
                    'bottlenecks': self._find_bottlenecks(),
                    'orphans': self._find_orphans(),
                    'density': nx.density(self.networkx_graph) if isinstance(self.networkx_graph, nx.Graph) else 0,
                    'is_dag': nx.is_directed_acyclic_graph(self.networkx_graph) if isinstance(self.networkx_graph, nx.DiGraph) else False
                }

                # Find strongly connected components if not DAG
                if isinstance(self.networkx_graph, nx.DiGraph) and not analysis['is_dag']:
                    try:
                        cycles = list(nx.simple_cycles(self.networkx_graph))
                        analysis['cycles'] = len(cycles)
                        analysis['cycle_details'] = cycles[:5]  # First 5 cycles
                    except Exception:
                        analysis['cycles'] = 0

                self.results['networkx_analysis'] = analysis
                logger.info(f"NetworkX analysis complete: {analysis['nodes']} nodes, {analysis['edges']} edges")
            else:
                logger.warning("NetworkX graph is empty, skipping analysis")
        except ImportError:
            logger.warning("NetworkX not available, install with: pip install networkx")
        except Exception as e:
            logger.warning(f"NetworkX analysis failed: {e}")

    def _store_todo2_results(self, analysis_results: Dict, insights: str) -> None:
        """Store results in Todo2 task."""
        if not self.todo2_task:
            return

        try:
            todo2_path = self.project_root / '.todo2' / 'state.todo2.json'
            if todo2_path.exists():
                with open(todo2_path, 'r') as f:
                    todo2_data = json.load(f)

                # Find task
                for task in todo2_data.get('todos', []):
                    if task['id'] == self.todo2_task['id']:
                        # Add result comment
                        if 'comments' not in task:
                            task['comments'] = []

                        result_comment = {
                            'id': f"{task['id']}-C-{len(task['comments']) + 1}",
                            'todoId': task['id'],
                            'type': 'result',
                            'content': f"**Automation Results:**\n\n{insights}\n\n**Key Findings:**\n{self._format_findings(analysis_results)}",
                            'created': datetime.now().isoformat(),
                            'lastModified': datetime.now().isoformat()
                        }

                        task['comments'].append(result_comment)
                        break

                # Save back
                with open(todo2_path, 'w') as f:
                    json.dump(todo2_data, f, indent=2)

                logger.info("Results stored in Todo2")
        except Exception as e:
            logger.warning(f"Failed to store Todo2 results: {e}")

    def _create_followup_tasks(self, analysis_results: Dict) -> None:
        """Create follow-up tasks based on findings."""
        if not self.todo2_task:
            return

        followup_tasks = self._identify_followup_tasks(analysis_results)

        if not followup_tasks:
            return

        try:
            todo2_path = self.project_root / '.todo2' / 'state.todo2.json'
            if todo2_path.exists():
                with open(todo2_path, 'r') as f:
                    todo2_data = json.load(f)

                for followup in followup_tasks:
                    task = {
                        'id': f"T-{datetime.now().strftime('%Y%m%d%H%M%S')}-{len(todo2_data.get('todos', []))}",
                        'name': followup['name'],
                        'content': followup.get('description', followup['name']),
                        'status': 'todo',
                        'priority': followup.get('priority', 'medium'),
                        'tags': followup.get('tags', ['automation', 'followup']),
                        'dependencies': [self.todo2_task['id']] if self.todo2_task else [],
                        'created': datetime.now().isoformat(),
                        'lastModified': datetime.now().isoformat()
                    }

                    if 'todos' not in todo2_data:
                        todo2_data['todos'] = []

                    todo2_data['todos'].append(task)
                    self.results['followup_tasks'].append(task['id'])

                # Save back
                with open(todo2_path, 'w') as f:
                    json.dump(todo2_data, f, indent=2)

                logger.info(f"Created {len(followup_tasks)} follow-up tasks")
        except Exception as e:
            logger.warning(f"Failed to create follow-up tasks: {e}")

    def _update_todo2_complete(self) -> None:
        """Update Todo2 task as complete."""
        if not self.todo2_task:
            return

        try:
            todo2_path = self.project_root / '.todo2' / 'state.todo2.json'
            if todo2_path.exists():
                with open(todo2_path, 'r') as f:
                    todo2_data = json.load(f)

                for task in todo2_data.get('todos', []):
                    if task['id'] == self.todo2_task['id']:
                        task['status'] = 'done'
                        task['lastModified'] = datetime.now().isoformat()
                        break

                with open(todo2_path, 'w') as f:
                    json.dump(todo2_data, f, indent=2)

                logger.info("Todo2 task marked as complete")
        except Exception as e:
            logger.warning(f"Failed to update Todo2 task: {e}")

    def _update_todo2_error(self, error: Exception) -> None:
        """Update Todo2 task with error."""
        if not self.todo2_task:
            return

        try:
            todo2_path = self.project_root / '.todo2' / 'state.todo2.json'
            if todo2_path.exists():
                with open(todo2_path, 'r') as f:
                    todo2_data = json.load(f)

                for task in todo2_data.get('todos', []):
                    if task['id'] == self.todo2_task['id']:
                        task['status'] = 'todo'
                        task['lastModified'] = datetime.now().isoformat()

                        if 'comments' not in task:
                            task['comments'] = []

                        error_comment = {
                            'id': f"{task['id']}-C-{len(task['comments']) + 1}",
                            'todoId': task['id'],
                            'type': 'note',
                            'content': f"**Error:** {str(error)}",
                            'created': datetime.now().isoformat(),
                            'lastModified': datetime.now().isoformat()
                        }

                        task['comments'].append(error_comment)
                        break

                with open(todo2_path, 'w') as f:
                    json.dump(todo2_data, f, indent=2)
        except Exception as e:
            logger.warning(f"Failed to update Todo2 task with error: {e}")

    # Abstract methods to be implemented by subclasses

    @abstractmethod
    def _get_tractatus_concept(self) -> str:
        """Return the concept to analyze with Tractatus Thinking."""
        pass

    @abstractmethod
    def _get_sequential_problem(self) -> str:
        """Return the problem to solve with Sequential Thinking."""
        pass

    @abstractmethod
    def _execute_analysis(self) -> Dict:
        """Execute the actual analysis - implemented by subclasses."""
        pass

    @abstractmethod
    def _generate_insights(self, analysis_results: Dict) -> str:
        """Generate insights from analysis results."""
        pass

    @abstractmethod
    def _generate_report(self, analysis_results: Dict, insights: str) -> str:
        """Generate final report."""
        pass

    # Helper methods with default implementations

    def _extract_components_from_concept(self, concept: str) -> List[str]:
        """Extract atomic components from concept (simplified Tractatus)."""
        # This is a simplified version - real implementation would use MCP server
        keywords = ['automation', 'analysis', 'validation', 'monitoring', 'tracking', 'synchronization']
        return [kw for kw in keywords if kw in concept.lower()]

    def _identify_dependencies(self, concept: str) -> List[str]:
        """Identify dependencies (simplified)."""
        return []

    def _plan_workflow_steps(self, problem: str) -> List[str]:
        """Plan workflow steps (simplified Sequential)."""
        return [
            "Load and analyze data",
            "Identify patterns and opportunities",
            "Generate recommendations",
            "Create follow-up tasks"
        ]

    def _needs_networkx(self) -> bool:
        """Determine if NetworkX analysis is needed."""
        return False

    def _build_networkx_graph(self):
        """Build NetworkX graph - override if needed."""
        return None

    def _find_critical_path(self) -> List[str]:
        """Find critical path in graph."""
        if not self.networkx_graph:
            return []

        try:
            import networkx as nx

            if not isinstance(self.networkx_graph, nx.DiGraph):
                return []

            # Find longest path (critical path)
            sources = [n for n in self.networkx_graph.nodes() if self.networkx_graph.in_degree(n) == 0]
            sinks = [n for n in self.networkx_graph.nodes() if self.networkx_graph.out_degree(n) == 0]

            longest_path = []
            for source in sources:
                for sink in sinks:
                    try:
                        paths = list(nx.all_simple_paths(self.networkx_graph, source, sink))
                        if paths:
                            longest = max(paths, key=len)
                            if len(longest) > len(longest_path):
                                longest_path = longest
                    except (nx.NetworkXNoPath, nx.NetworkXError):
                        continue

            return longest_path
        except Exception as e:
            logger.warning(f"Critical path analysis failed: {e}")
            return []

    def _find_bottlenecks(self) -> List[str]:
        """Find bottlenecks in graph (nodes with high out-degree)."""
        if not self.networkx_graph:
            return []

        try:
            import networkx as nx

            # Find nodes with high out-degree (many dependents)
            out_degrees = dict(self.networkx_graph.out_degree())
            if not out_degrees:
                return []

            # Nodes with out-degree > 3 are potential bottlenecks
            threshold = max(3, max(out_degrees.values()) * 0.3) if out_degrees.values() else 3
            bottlenecks = [node for node, degree in out_degrees.items() if degree >= threshold]

            return sorted(bottlenecks, key=lambda n: out_degrees[n], reverse=True)[:10]
        except Exception as e:
            logger.warning(f"Bottleneck analysis failed: {e}")
            return []

    def _find_orphans(self) -> List[str]:
        """Find orphaned nodes in graph (no incoming edges)."""
        if not self.networkx_graph:
            return []

        try:
            import networkx as nx

            if isinstance(self.networkx_graph, nx.DiGraph):
                # Nodes with no incoming edges
                orphans = [n for n in self.networkx_graph.nodes() if self.networkx_graph.in_degree(n) == 0]
            else:
                # Nodes with no edges
                orphans = [n for n in self.networkx_graph.nodes() if self.networkx_graph.degree(n) == 0]

            return orphans
        except Exception as e:
            logger.warning(f"Orphan analysis failed: {e}")
            return []

    def _identify_followup_tasks(self, analysis_results: Dict) -> List[Dict]:
        """Identify follow-up tasks from analysis results."""
        return []

    def _format_findings(self, analysis_results: Dict) -> str:
        """Format findings for Todo2 comment."""
        return json.dumps(analysis_results, indent=2)

    def _fallback_component_extraction(self, concept: str) -> List[str]:
        """Fallback component extraction."""
        return self._extract_components_from_concept(concept)

    def _fallback_workflow_steps(self) -> List[str]:
        """Fallback workflow steps."""
        return self._plan_workflow_steps("")
