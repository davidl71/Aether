#!/usr/bin/env python3
"""
NetworkX-based project analysis tool for Todo2 tasks, documentation, and architecture.

This tool creates graph representations of:
1. Todo2 task dependencies
2. Documentation cross-references
3. Architecture component relationships

Usage:
    python python/tools/project_analyzer.py --tasks
    python python/tools/project_analyzer.py --docs
    python python/tools/project_analyzer.py --architecture
    python python/tools/project_analyzer.py --all
"""

import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path
from typing import Dict, List, Set, Tuple, Optional

try:
    import networkx as nx
except ImportError:
    print("ERROR: networkx is required. Install with: pip install networkx>=3.2.0")
    sys.exit(1)

# Project root directory
PROJECT_ROOT = Path(__file__).parent.parent.parent


class Todo2Analyzer:
    """Analyze Todo2 task dependencies and relationships."""

    def __init__(self, todo2_path: Path):
        self.todo2_path = todo2_path
        self.graph = nx.DiGraph()
        self.tasks = {}

    def load_tasks(self) -> Dict:
        """Load Todo2 tasks from JSON file."""
        if not self.todo2_path.exists():
            print(f"Warning: Todo2 file not found: {self.todo2_path}")
            return {}

        with open(self.todo2_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
            return {task['id']: task for task in data.get('todos', [])}

    def build_graph(self) -> nx.DiGraph:
        """Build dependency graph from Todo2 tasks."""
        self.tasks = self.load_tasks()

        # Add nodes (tasks)
        for task_id, task in self.tasks.items():
            self.graph.add_node(
                task_id,
                name=task.get('name', ''),
                status=task.get('status', 'Unknown'),
                priority=task.get('priority', 'medium'),
                tags=task.get('tags', []),
                created=task.get('created', ''),
                last_modified=task.get('lastModified', '')
            )

        # Add edges (dependencies)
        for task_id, task in self.tasks.items():
            dependencies = task.get('dependencies', [])
            for dep_id in dependencies:
                if dep_id in self.tasks:
                    self.graph.add_edge(dep_id, task_id, relationship='depends_on')

        return self.graph

    def analyze_critical_path(self) -> List[str]:
        """Find critical path (longest dependency chain)."""
        if not self.graph.nodes():
            return []

        # Find all paths from source nodes (no dependencies) to sink nodes (no dependents)
        sources = [n for n in self.graph.nodes() if self.graph.in_degree(n) == 0]
        sinks = [n for n in self.graph.nodes() if self.graph.out_degree(n) == 0]

        longest_path = []
        for source in sources:
            for sink in sinks:
                try:
                    paths = list(nx.all_simple_paths(self.graph, source, sink))
                    if paths:
                        longest = max(paths, key=len)
                        if len(longest) > len(longest_path):
                            longest_path = longest
                except nx.NetworkXNoPath:
                    continue

        return longest_path

    def find_bottlenecks(self, top_n: int = 10) -> List[Tuple[str, int]]:
        """Find tasks with most dependencies (potential bottlenecks)."""
        in_degrees = [(node, self.graph.in_degree(node)) for node in self.graph.nodes()]
        return sorted(in_degrees, key=lambda x: x[1], reverse=True)[:top_n]

    def find_isolated_tasks(self) -> List[str]:
        """Find tasks with no dependencies or dependents."""
        isolated = []
        for node in self.graph.nodes():
            if self.graph.in_degree(node) == 0 and self.graph.out_degree(node) == 0:
                isolated.append(node)
        return isolated

    def analyze_by_status(self) -> Dict[str, int]:
        """Count tasks by status."""
        status_counts = defaultdict(int)
        for node in self.graph.nodes():
            status = self.graph.nodes[node].get('status', 'Unknown')
            status_counts[status] += 1
        return dict(status_counts)

    def analyze_by_tag(self) -> Dict[str, int]:
        """Count tasks by tag."""
        tag_counts = defaultdict(int)
        for node in self.graph.nodes():
            tags = self.graph.nodes[node].get('tags', [])
            for tag in tags:
                tag_counts[tag] += 1
        return dict(sorted(tag_counts.items(), key=lambda x: x[1], reverse=True))

    def get_insights(self) -> Dict:
        """Generate comprehensive insights."""
        return {
            'total_tasks': len(self.graph.nodes()),
            'total_dependencies': len(self.graph.edges()),
            'critical_path': self.analyze_critical_path(),
            'critical_path_length': len(self.analyze_critical_path()),
            'bottlenecks': self.find_bottlenecks(10),
            'isolated_tasks': self.find_isolated_tasks(),
            'status_distribution': self.analyze_by_status(),
            'tag_distribution': self.analyze_by_tag(),
            'is_dag': nx.is_directed_acyclic_graph(self.graph),
            'strongly_connected_components': list(nx.strongly_connected_components(self.graph)),
        }


class DocumentationAnalyzer:
    """Analyze documentation cross-references and structure."""

    def __init__(self, docs_dir: Path):
        self.docs_dir = docs_dir
        self.graph = nx.DiGraph()
        self.docs = {}

    def find_doc_files(self) -> List[Path]:
        """Find all markdown documentation files."""
        return list(self.docs_dir.rglob('*.md'))

    def extract_references(self, content: str) -> Set[str]:
        """Extract markdown link references from content."""
        # Match markdown links: [text](path) or [text](path#anchor)
        pattern = r'\[([^\]]+)\]\(([^)]+)\)'
        matches = re.findall(pattern, content)
        return {match[1] for match in matches}

    def normalize_path(self, doc_path: Path, ref: str) -> Optional[Path]:
        """Normalize reference path relative to document."""
        # Remove anchors
        ref = ref.split('#')[0]
        if not ref:
            return None

        # Handle absolute paths
        if ref.startswith('/'):
            ref = ref[1:]

        # Try relative to current document
        ref_path = (doc_path.parent / ref).resolve()
        if ref_path.exists() and ref_path.is_file():
            return ref_path

        # Try relative to docs directory
        ref_path = (self.docs_dir / ref).resolve()
        if ref_path.exists() and ref_path.is_file():
            return ref_path

        return None

    def build_graph(self) -> nx.DiGraph:
        """Build graph of documentation cross-references."""
        doc_files = self.find_doc_files()

        # Add nodes (documents)
        for doc_path in doc_files:
            rel_path = doc_path.relative_to(PROJECT_ROOT)
            self.graph.add_node(
                str(rel_path),
                path=str(doc_path),
                name=doc_path.name,
                size=doc_path.stat().st_size
            )

        # Add edges (references)
        for doc_path in doc_files:
            rel_path = doc_path.relative_to(PROJECT_ROOT)
            try:
                content = doc_path.read_text(encoding='utf-8')
                references = self.extract_references(content)

                for ref in references:
                    target_path = self.normalize_path(doc_path, ref)
                    if target_path:
                        target_rel = target_path.relative_to(PROJECT_ROOT)
                        if str(target_rel) in self.graph:
                            self.graph.add_edge(
                                str(rel_path),
                                str(target_rel),
                                relationship='references'
                            )
            except Exception as e:
                print(f"Warning: Error processing {doc_path}: {e}")

        return self.graph

    def find_central_docs(self, top_n: int = 10) -> List[Tuple[str, int]]:
        """Find most referenced documents (high in-degree centrality)."""
        in_degrees = [(node, self.graph.in_degree(node)) for node in self.graph.nodes()]
        return sorted(in_degrees, key=lambda x: x[1], reverse=True)[:top_n]

    def find_isolated_docs(self) -> List[str]:
        """Find documents with no references or references to them."""
        isolated = []
        for node in self.graph.nodes():
            if self.graph.in_degree(node) == 0 and self.graph.out_degree(node) == 0:
                isolated.append(node)
        return isolated

    def get_insights(self) -> Dict:
        """Generate comprehensive insights."""
        return {
            'total_docs': len(self.graph.nodes()),
            'total_references': len(self.graph.edges()),
            'central_docs': self.find_central_docs(10),
            'isolated_docs': self.find_isolated_docs(),
            'avg_references_per_doc': len(self.graph.edges()) / max(len(self.graph.nodes()), 1),
        }


class ArchitectureAnalyzer:
    """Analyze codebase architecture and component relationships."""

    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.graph = nx.DiGraph()

    def find_source_files(self) -> List[Path]:
        """Find source files by language."""
        patterns = {
            'cpp': '**/*.cpp',
            'h': '**/*.h',
            'py': '**/*.py',
            'rs': '**/*.rs',
            'ts': '**/*.ts',
            'tsx': '**/*.tsx',
        }
        files = []
        for _lang, pattern in patterns.items():
            files.extend(self.project_root.glob(pattern))
        return files

    def extract_imports(self, file_path: Path) -> Set[str]:
        """Extract imports/includes from source file."""
        imports = set()
        try:
            content = file_path.read_text(encoding='utf-8')
        except Exception:
            return imports

        # C++ includes
        if file_path.suffix in ['.cpp', '.h', '.hpp']:
            pattern = r'#include\s+[<"]([^>"]+)[>"]'
            matches = re.findall(pattern, content)
            imports.update(matches)

        # Python imports
        elif file_path.suffix == '.py':
            pattern = r'^(?:from\s+(\S+)\s+)?import\s+(\S+)'
            matches = re.findall(pattern, content, re.MULTILINE)
            for match in matches:
                if match[0]:
                    imports.add(match[0])
                if match[1]:
                    imports.add(match[1].split('.')[0])

        # TypeScript/JavaScript imports
        elif file_path.suffix in ['.ts', '.tsx', '.js', '.jsx']:
            pattern = r'import\s+(?:.*\s+from\s+)?[\'"]([^\'"]+)[\'"]'
            matches = re.findall(pattern, content)
            imports.update(matches)

        return imports

    def build_graph(self) -> nx.DiGraph:
        """Build graph of file dependencies."""
        source_files = self.find_source_files()

        # Add nodes (files)
        for file_path in source_files:
            rel_path = file_path.relative_to(self.project_root)
            self.graph.add_node(
                str(rel_path),
                path=str(file_path),
                name=file_path.name,
                language=file_path.suffix[1:],
                size=file_path.stat().st_size if file_path.exists() else 0
            )

        # Add edges (imports/includes)
        for file_path in source_files:
            rel_path = file_path.relative_to(self.project_root)
            imports = self.extract_imports(file_path)

            for imp in imports:
                # Try to find matching file
                # This is simplified - real implementation would need better matching
                for node in self.graph.nodes():
                    if imp in node or node.endswith(imp):
                        if node != str(rel_path):
                            self.graph.add_edge(
                                str(rel_path),
                                node,
                                relationship='imports'
                            )

        return self.graph

    def get_insights(self) -> Dict:
        """Generate comprehensive insights."""
        return {
            'total_files': len(self.graph.nodes()),
            'total_dependencies': len(self.graph.edges()),
            'avg_dependencies_per_file': len(self.graph.edges()) / max(len(self.graph.nodes()), 1),
        }


def print_insights(title: str, insights: Dict):
    """Print formatted insights."""
    print(f"\n{'='*60}")
    print(f"{title}")
    print(f"{'='*60}\n")

    for key, value in insights.items():
        if isinstance(value, list):
            print(f"{key}:")
            for item in value[:10]:  # Limit to top 10
                print(f"  - {item}")
        elif isinstance(value, dict):
            print(f"{key}:")
            for k, v in sorted(value.items(), key=lambda x: x[1] if isinstance(x[1], (int, float)) else 0, reverse=True)[:10]:
                print(f"  {k}: {v}")
        else:
            print(f"{key}: {value}")


def main():
    parser = argparse.ArgumentParser(description='Analyze project using NetworkX')
    parser.add_argument('--tasks', action='store_true', help='Analyze Todo2 tasks')
    parser.add_argument('--docs', action='store_true', help='Analyze documentation')
    parser.add_argument('--architecture', action='store_true', help='Analyze architecture')
    parser.add_argument('--all', action='store_true', help='Run all analyses')
    parser.add_argument('--todo2-path', type=Path, default=PROJECT_ROOT / '.todo2' / 'state.todo2.json',
                       help='Path to Todo2 JSON file')
    parser.add_argument('--docs-dir', type=Path, default=PROJECT_ROOT / 'docs',
                       help='Path to documentation directory')

    args = parser.parse_args()

    if not any([args.tasks, args.docs, args.architecture, args.all]):
        parser.print_help()
        return

    if args.all or args.tasks:
        print("\n" + "="*60)
        print("TODO2 TASK ANALYSIS")
        print("="*60)
        analyzer = Todo2Analyzer(args.todo2_path)
        analyzer.build_graph()
        insights = analyzer.get_insights()
        print_insights("Todo2 Task Insights", insights)

    if args.all or args.docs:
        print("\n" + "="*60)
        print("DOCUMENTATION ANALYSIS")
        print("="*60)
        analyzer = DocumentationAnalyzer(args.docs_dir)
        analyzer.build_graph()
        insights = analyzer.get_insights()
        print_insights("Documentation Insights", insights)

    if args.all or args.architecture:
        print("\n" + "="*60)
        print("ARCHITECTURE ANALYSIS")
        print("="*60)
        analyzer = ArchitectureAnalyzer(PROJECT_ROOT)
        analyzer.build_graph()
        insights = analyzer.get_insights()
        print_insights("Architecture Insights", insights)


if __name__ == '__main__':
    main()
