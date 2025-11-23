"""
Unit Tests for MCP Tool Wrappers

Tests each tool wrapper to ensure proper error handling and response formatting.
"""

import json
import pytest
from unittest.mock import Mock, patch, MagicMock
from pathlib import Path
import sys

# Add project root to path
project_root = Path(__file__).parent.parent.parent.parent
sys.path.insert(0, str(project_root))
sys.path.insert(0, str(Path(__file__).parent.parent))


class TestDocumentationHealthTool:
    """Tests for check_documentation_health tool."""

    @patch('mcp_servers.project_management_automation.tools.docs_health.DocumentationHealthAnalyzerV2')
    def test_check_documentation_health_success(self, mock_analyzer_class):
        """Test successful documentation health check."""
        from mcp_servers.project_management_automation.tools.docs_health import check_documentation_health

        # Mock analyzer
        mock_analyzer = Mock()
        mock_analyzer.run.return_value = {
            'status': 'success',
            'results': {
                'health_score': 85,
                'link_validation': {
                    'total_links': 100,
                    'broken_internal': [],
                    'broken_external': []
                },
                'format_validation': {
                    'format_errors': []
                }
            },
            'followup_tasks': []
        }
        mock_analyzer_class.return_value = mock_analyzer

        # Call tool
        result = check_documentation_health(output_path="test_report.md", create_tasks=True)
        result_data = json.loads(result)

        # Assertions
        assert result_data['success'] is True
        assert result_data['data']['health_score'] == 85
        assert result_data['data']['report_path'] is not None

    @patch('mcp_servers.project_management_automation.tools.docs_health.DocumentationHealthAnalyzerV2')
    def test_check_documentation_health_error(self, mock_analyzer_class):
        """Test error handling in documentation health check."""
        from mcp_servers.project_management_automation.tools.docs_health import check_documentation_health

        # Mock analyzer to raise exception
        mock_analyzer_class.side_effect = Exception("Test error")

        # Call tool
        result = check_documentation_health()
        result_data = json.loads(result)

        # Assertions
        assert result_data['success'] is False
        assert 'error' in result_data


class TestTodo2AlignmentTool:
    """Tests for analyze_todo2_alignment tool."""

    @patch('scripts.automate_todo2_alignment_v2.Todo2AlignmentAnalyzerV2')
    def test_analyze_todo2_alignment_success(self, mock_analyzer_class):
        """Test successful Todo2 alignment analysis."""
        from mcp_servers.project_management_automation.tools.todo2_alignment import analyze_todo2_alignment

        # Mock analyzer
        mock_analyzer = Mock()
        mock_analyzer.run.return_value = {
            'status': 'success',
            'results': {
                'tasks_analyzed': 50,
                'misaligned_tasks': [],
                'alignment_scores': {
                    'average': 0.85
                }
            },
            'followup_tasks': []
        }
        mock_analyzer_class.return_value = mock_analyzer

        # Call tool
        result = analyze_todo2_alignment(create_followup_tasks=True)
        result_data = json.loads(result)

        # Assertions
        assert result_data['success'] is True
        assert result_data['data']['total_tasks_analyzed'] == 50


class TestDuplicateDetectionTool:
    """Tests for detect_duplicate_tasks tool."""

    @patch('scripts.automate_todo2_duplicate_detection.Todo2DuplicateDetector')
    def test_detect_duplicate_tasks_success(self, mock_detector_class):
        """Test successful duplicate detection."""
        from mcp_servers.project_management_automation.tools.duplicate_detection import detect_duplicate_tasks

        # Mock detector
        mock_detector = Mock()
        mock_detector.run.return_value = {
            'status': 'success',
            'results': {
                'duplicates': {
                    'duplicate_ids': [],
                    'exact_name_matches': [],
                    'similar_name_matches': [],
                    'similar_description_matches': [],
                    'self_dependencies': []
                }
            }
        }
        mock_detector_class.return_value = mock_detector

        # Call tool
        result = detect_duplicate_tasks(similarity_threshold=0.85, auto_fix=False)
        result_data = json.loads(result)

        # Assertions
        assert result_data['success'] is True
        assert result_data['data']['total_duplicates_found'] == 0


class TestDependencySecurityTool:
    """Tests for scan_dependency_security tool."""

    @patch('scripts.automate_dependency_security.DependencySecurityAnalyzer')
    def test_scan_dependency_security_success(self, mock_analyzer_class):
        """Test successful dependency security scan."""
        from mcp_servers.project_management_automation.tools.dependency_security import scan_dependency_security

        # Mock analyzer
        mock_analyzer = Mock()
        mock_analyzer.run.return_value = {
            'status': 'success',
            'summary': {
                'total_vulnerabilities': 0,
                'by_severity': {},
                'by_language': {},
                'critical_vulnerabilities': []
            },
            'python': [],
            'rust': [],
            'npm': []
        }
        mock_analyzer.output_file = Path('test_report.md')
        mock_analyzer_class.return_value = mock_analyzer

        # Call tool
        result = scan_dependency_security(languages=['python'])
        result_data = json.loads(result)

        # Assertions
        assert result_data['success'] is True
        assert result_data['data']['total_vulnerabilities'] == 0


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
