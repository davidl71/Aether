#!/usr/bin/env python3
"""
Automated Documentation Health Monitoring Script

This script automates documentation health checks by:
1. Validating all links (internal/external)
2. Checking format compliance
3. Verifying content completeness
4. Checking date currency
5. Validating cross-references
6. Tracking trends over time
7. Generating comprehensive health report

Usage:
    python3 scripts/automate_docs_health.py [--config config.json] [--output docs/DOCUMENTATION_HEALTH_REPORT.md]

Configuration:
    See scripts/docs_health_config.json for configuration options.
"""

import argparse
import json
import logging
import os
import re
import subprocess
import sys
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple
from urllib.parse import urlparse
from urllib.request import urlopen, Request
from urllib.error import URLError, HTTPError

# ANSI escape sequence pattern
ANSI_ESCAPE = re.compile(r'\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])')

# Add project root to path
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler(project_root / 'scripts' / 'docs_health.log'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)


class DocumentationHealthAnalyzer:
    """Analyzes documentation health across multiple dimensions."""

    def __init__(self, config: Dict):
        self.config = config
        self.project_root = project_root
        self.docs_path = self.project_root / 'docs'
        self.history_path = self.project_root / 'scripts' / '.docs_health_history.json'

        # Load history
        self.history = self._load_history()

        # Analysis results
        self.results = {
            'timestamp': datetime.now().isoformat(),
            'link_validation': {
                'total_links': 0,
                'internal_links': 0,
                'external_links': 0,
                'broken_internal': [],
                'broken_external': [],
                'skipped': 0
            },
            'format_validation': {
                'files_checked': 0,
                'format_errors': [],
                'missing_required_fields': [],
                'missing_recommended_fields': []
            },
            'date_currency': {
                'files_with_dates': 0,
                'stale_files': [],
                'missing_dates': []
            },
            'cross_references': {
                'total_references': 0,
                'broken_references': [],
                'orphaned_files': []
            }
        }

    def _strip_ansi_codes(self, text: str) -> str:
        """Strip ANSI escape sequences from text."""
        return ANSI_ESCAPE.sub('', text)

    def _load_history(self) -> Dict:
        """Load historical health data."""
        if self.history_path.exists():
            try:
                with open(self.history_path, 'r') as f:
                    return json.load(f)
            except json.JSONDecodeError:
                logger.warning("Invalid history file, starting fresh")
        return {'runs': []}

    def _save_history(self):
        """Save current run to history."""
        if 'runs' not in self.history:
            self.history['runs'] = []

        # Keep last 50 runs
        self.history['runs'].append(self.results)
        if len(self.history['runs']) > 50:
            self.history['runs'] = self.history['runs'][-50:]

        with open(self.history_path, 'w') as f:
            json.dump(self.history, f, indent=2)

    def validate_links(self) -> None:
        """Validate all links in documentation."""
        logger.info("Validating links...")

        # Find all markdown files
        md_files = list(self.docs_path.rglob('*.md'))

        # Skip certain directories
        skip_dirs = {'archive', 'indices', 'message_schemas', 'resource-summaries', 'video-summaries'}
        md_files = [f for f in md_files if not any(skip in str(f) for skip in skip_dirs)]

        link_pattern = re.compile(r'\[([^\]]+)\]\(([^)]+)\)')
        angle_bracket_pattern = re.compile(r'<https?://[^>]+>')

        all_internal_files = {f.relative_to(self.docs_path) for f in md_files}

        for md_file in md_files:
            try:
                content = md_file.read_text(encoding='utf-8')
                rel_path = md_file.relative_to(self.docs_path)

                # Extract markdown links
                for match in link_pattern.finditer(content):
                    url = match.group(2)
                    self._validate_link(url, md_file, match.start(), all_internal_files)

                # Extract angle bracket URLs
                for match in angle_bracket_pattern.finditer(content):
                    url = match.group(0)[1:-1]  # Remove < >
                    self._validate_link(url, md_file, match.start(), all_internal_files)

            except Exception as e:
                logger.error(f"Error processing {md_file}: {e}")

    def _validate_link(self, url: str, source_file: Path, position: int,
                      all_internal_files: Set[Path]) -> None:
        """Validate a single link."""
        self.results['link_validation']['total_links'] += 1

        # Skip patterns
        skip_patterns = ['mailto:', '#', 'docs/', 'github.com.*blob']
        if any(pattern in url for pattern in skip_patterns):
            self.results['link_validation']['skipped'] += 1
            return

        # Check if internal link
        if not url.startswith('http://') and not url.startswith('https://'):
            # Internal link
            self.results['link_validation']['internal_links'] += 1

            # Resolve relative path
            source_dir = source_file.parent
            target_path = (source_dir / url).resolve()

            # Check if file exists
            if not target_path.exists():
                self.results['link_validation']['broken_internal'].append({
                    'url': url,
                    'source': str(source_file.relative_to(self.project_root)),
                    'position': position
                })
        else:
            # External link
            self.results['link_validation']['external_links'] += 1

            # Check URL accessibility (with timeout)
            try:
                req = Request(url, headers={'User-Agent': 'Mozilla/5.0'})
                with urlopen(req, timeout=10) as response:
                    status = response.getcode()
                    if status not in [200, 301, 302, 303, 307, 308]:
                        raise HTTPError(url, status, "Non-success status", None, None)
            except (URLError, HTTPError, TimeoutError) as e:
                self.results['link_validation']['broken_external'].append({
                    'url': url,
                    'source': str(source_file.relative_to(self.project_root)),
                    'position': position,
                    'error': str(e)
                })

    def validate_format(self) -> None:
        """Validate documentation format compliance."""
        logger.info("Validating format compliance...")

        # Check API_DOCUMENTATION_INDEX.md using existing script
        index_file = self.docs_path / 'API_DOCUMENTATION_INDEX.md'
        if index_file.exists():
            try:
                result = subprocess.run(
                    [sys.executable, str(self.project_root / 'scripts' / 'validate_docs_format.py')],
                    capture_output=True,
                    text=True,
                    cwd=self.project_root
                )

                if result.returncode != 0:
                    # Parse errors from output (strip ANSI codes)
                    for line in result.stdout.split('\n'):
                        if 'Error' in line or 'Missing' in line:
                            # Strip ANSI escape sequences
                            clean_line = self._strip_ansi_codes(line)
                            if clean_line.strip():  # Only add non-empty lines
                                self.results['format_validation']['format_errors'].append(clean_line)

                self.results['format_validation']['files_checked'] += 1
            except Exception as e:
                logger.error(f"Error running format validation: {e}")

        # Check for required sections in key documents
        key_docs = [
            'README.md',
            'docs/INVESTMENT_STRATEGY_FRAMEWORK.md',
            'docs/PRIMARY_GOALS_AND_REQUIREMENTS.md'
        ]

        for doc_path in key_docs:
            full_path = self.project_root / doc_path
            if full_path.exists():
                self._check_required_sections(full_path)

    def _check_required_sections(self, file_path: Path) -> None:
        """Check for required sections in a document."""
        content = file_path.read_text(encoding='utf-8')

        # Basic section check (can be enhanced)
        if file_path.name == 'README.md':
            required = ['##', 'Overview', 'Installation']
            for req in required:
                if req not in content:
                    self.results['format_validation']['missing_required_fields'].append({
                        'file': str(file_path.relative_to(self.project_root)),
                        'missing': req
                    })

    def check_date_currency(self) -> None:
        """Check documentation currency (last updated dates)."""
        logger.info("Checking date currency...")

        date_patterns = [
            re.compile(r'\*\*Last Updated\*\*:\s*(\d{4}-\d{2}-\d{2})', re.IGNORECASE),
            re.compile(r'Last Updated:\s*(\d{4}-\d{2}-\d{2})', re.IGNORECASE),
            re.compile(r'Date:\s*(\d{4}-\d{2}-\d{2})', re.IGNORECASE),
        ]

        stale_threshold_days = self.config.get('stale_threshold_days', 90)
        threshold_date = datetime.now() - timedelta(days=stale_threshold_days)

        # Key documents that should have dates
        key_docs = list(self.docs_path.glob('*.md'))

        for doc_file in key_docs:
            try:
                content = doc_file.read_text(encoding='utf-8')
                date_found = False

                for pattern in date_patterns:
                    match = pattern.search(content)
                    if match:
                        date_found = True
                        date_str = match.group(1)
                        try:
                            doc_date = datetime.strptime(date_str, '%Y-%m-%d')
                            self.results['date_currency']['files_with_dates'] += 1

                            if doc_date < threshold_date:
                                self.results['date_currency']['stale_files'].append({
                                    'file': str(doc_file.relative_to(self.project_root)),
                                    'last_updated': date_str,
                                    'days_old': (datetime.now() - doc_date).days
                                })
                        except ValueError:
                            pass
                        break

                # Check if key doc should have date but doesn't
                if not date_found and doc_file.name in ['README.md', 'INVESTMENT_STRATEGY_FRAMEWORK.md']:
                    self.results['date_currency']['missing_dates'].append({
                        'file': str(doc_file.relative_to(self.project_root))
                    })

            except Exception as e:
                logger.error(f"Error checking date in {doc_file}: {e}")

    def validate_cross_references(self) -> None:
        """Validate cross-references between documents."""
        logger.info("Validating cross-references...")

        # Find all markdown files
        md_files = list(self.docs_path.rglob('*.md'))
        skip_dirs = {'archive', 'indices', 'message_schemas', 'resource-summaries', 'video-summaries'}
        md_files = [f for f in md_files if not any(skip in str(f) for skip in skip_dirs)]

        # Build reference map
        referenced_files: Set[str] = set()
        file_references: Dict[str, List[str]] = {}

        for md_file in md_files:
            try:
                content = md_file.read_text(encoding='utf-8')
                rel_path = str(md_file.relative_to(self.docs_path))
                file_references[rel_path] = []

                # Find references to other docs
                link_pattern = re.compile(r'\[([^\]]+)\]\(([^)]+)\)')
                for match in link_pattern.finditer(content):
                    url = match.group(2)

                    # Check if internal doc reference
                    if not url.startswith('http') and not url.startswith('mailto') and not url.startswith('#'):
                        # Resolve path (handle both relative to docs and project root)
                        source_dir = md_file.parent
                        target_path = (source_dir / url).resolve()

                        # Check if target exists and is a markdown file
                        if target_path.exists() and target_path.suffix == '.md':
                            # Try to get relative path to docs directory
                            try:
                                target_rel = str(target_path.relative_to(self.docs_path))
                                referenced_files.add(target_rel)
                                file_references[rel_path].append(target_rel)
                                self.results['cross_references']['total_references'] += 1
                            except ValueError:
                                # Target is outside docs directory - check if it's in project root
                                try:
                                    target_rel_to_project = str(target_path.relative_to(self.project_root))
                                    # Track as valid reference even if outside docs
                                    self.results['cross_references']['total_references'] += 1
                                    # Don't add to referenced_files since it's outside docs
                                except ValueError:
                                    # Target is outside project root - mark as broken
                                    self.results['cross_references']['broken_references'].append({
                                        'source': rel_path,
                                        'target': url,
                                        'resolved': 'outside project root'
                                    })
                        else:
                            self.results['cross_references']['broken_references'].append({
                                'source': rel_path,
                                'target': url,
                                'resolved': str(target_path) if target_path.exists() else 'not found'
                            })

            except Exception as e:
                logger.error(f"Error processing {md_file}: {e}")

        # Find orphaned files (not referenced by any other doc)
        all_files = {str(f.relative_to(self.docs_path)) for f in md_files}
        orphaned = all_files - referenced_files

        # Filter out index files and READMEs (they're entry points)
        self.results['cross_references']['orphaned_files'] = [
            f for f in orphaned
            if not any(skip in f for skip in ['INDEX', 'README', 'SUMMARY', 'TEMPLATE'])
        ]

    def calculate_health_score(self) -> float:
        """Calculate overall documentation health score (0-100)."""
        total_issues = 0
        max_issues = 0

        # Link health (40% weight)
        link_issues = (
            len(self.results['link_validation']['broken_internal']) +
            len(self.results['link_validation']['broken_external'])
        )
        total_links = self.results['link_validation']['total_links']
        if total_links > 0:
            link_score = max(0, 1.0 - (link_issues / total_links))
        else:
            link_score = 1.0

        # Format health (20% weight)
        format_issues = len(self.results['format_validation']['format_errors'])
        format_score = max(0, 1.0 - (format_issues / 10.0))  # Normalize to 10 issues max

        # Date currency (20% weight)
        stale_issues = len(self.results['date_currency']['stale_files'])
        date_score = max(0, 1.0 - (stale_issues / 20.0))  # Normalize to 20 stale files max

        # Cross-reference health (20% weight)
        ref_issues = (
            len(self.results['cross_references']['broken_references']) +
            len(self.results['cross_references']['orphaned_files'])
        )
        ref_score = max(0, 1.0 - (ref_issues / 30.0))  # Normalize to 30 issues max

        # Weighted average
        health_score = (
            link_score * 0.4 +
            format_score * 0.2 +
            date_score * 0.2 +
            ref_score * 0.2
        ) * 100

        return round(health_score, 1)

    def generate_report(self, health_score: float) -> str:
        """Generate comprehensive health report."""
        timestamp = datetime.now().strftime('%Y-%m-%d %H:%M:%S')

        # Build sections
        broken_links_section = self._build_broken_links_section()
        format_issues_section = self._build_format_issues_section()
        stale_docs_section = self._build_stale_docs_section()
        cross_ref_issues_section = self._build_cross_ref_issues_section()
        trends_section = self._build_trends_section(health_score)

        report = f"""# Documentation Health Report

*Generated: {timestamp}*
*Generated By: Automated Documentation Health Script*

## Executive Summary

**Overall Health Score: {health_score}%** {'✅' if health_score >= 80 else '⚠️' if health_score >= 60 else '❌'}

**Key Metrics:**
- **Total Links Checked**: {self.results['link_validation']['total_links']}
- **Broken Links**: {len(self.results['link_validation']['broken_internal']) + len(self.results['link_validation']['broken_external'])}
- **Format Errors**: {len(self.results['format_validation']['format_errors'])}
- **Stale Documents**: {len(self.results['date_currency']['stale_files'])}
- **Cross-Reference Issues**: {len(self.results['cross_references']['broken_references']) + len(self.results['cross_references']['orphaned_files'])}

---

## Link Validation

**Summary:**
- Total Links: {self.results['link_validation']['total_links']}
- Internal Links: {self.results['link_validation']['internal_links']}
- External Links: {self.results['link_validation']['external_links']}
- Skipped: {self.results['link_validation']['skipped']}
- Broken Internal: {len(self.results['link_validation']['broken_internal'])}
- Broken External: {len(self.results['link_validation']['broken_external'])}

{broken_links_section}

---

## Format Validation

**Summary:**
- Files Checked: {self.results['format_validation']['files_checked']}
- Format Errors: {len(self.results['format_validation']['format_errors'])}
- Missing Required Fields: {len(self.results['format_validation']['missing_required_fields'])}

{format_issues_section}

---

## Date Currency

**Summary:**
- Files With Dates: {self.results['date_currency']['files_with_dates']}
- Stale Files (>90 days): {len(self.results['date_currency']['stale_files'])}
- Missing Dates: {len(self.results['date_currency']['missing_dates'])}

{stale_docs_section}

---

## Cross-Reference Validation

**Summary:**
- Total References: {self.results['cross_references']['total_references']}
- Broken References: {len(self.results['cross_references']['broken_references'])}
- Orphaned Files: {len(self.results['cross_references']['orphaned_files'])}

{cross_ref_issues_section}

---

## Trends

{trends_section}

---

## Recommendations

### Immediate Actions

1. **Fix Broken Links**
   - {len(self.results['link_validation']['broken_internal']) + len(self.results['link_validation']['broken_external'])} broken links need attention
   - Prioritize internal broken links first

2. **Update Stale Documents**
   - {len(self.results['date_currency']['stale_files'])} documents haven't been updated in 90+ days
   - Review and update or archive stale content

3. **Fix Format Issues**
   - {len(self.results['format_validation']['format_errors'])} format errors need correction
   - Ensure all API documentation entries follow the template

4. **Resolve Cross-Reference Issues**
   - {len(self.results['cross_references']['broken_references'])} broken references
   - {len(self.results['cross_references']['orphaned_files'])} orphaned files

### Health Score Breakdown

**Overall Score: {health_score}%**

**Components:**
- Link Health: {round((1.0 - (len(self.results['link_validation']['broken_internal']) + len(self.results['link_validation']['broken_external'])) / max(self.results['link_validation']['total_links'], 1)) * 100, 1)}%
- Format Health: {round((1.0 - len(self.results['format_validation']['format_errors']) / 10.0) * 100, 1)}%
- Date Currency: {round((1.0 - len(self.results['date_currency']['stale_files']) / 20.0) * 100, 1)}%
- Cross-Reference Health: {round((1.0 - (len(self.results['cross_references']['broken_references']) + len(self.results['cross_references']['orphaned_files'])) / 30.0) * 100, 1)}%

**Target: 80%+ health score**

---

## Next Steps

1. Review this report
2. Fix broken links
3. Update stale documents
4. Correct format issues
5. Resolve cross-reference problems
6. Re-run analysis to track improvement

---

*This report was automatically generated. Review and fix issues as needed.*
"""
        return report

    def _build_broken_links_section(self) -> str:
        """Build broken links section."""
        if not self.results['link_validation']['broken_internal'] and not self.results['link_validation']['broken_external']:
            return "✅ **No broken links found!**"

        section = ""
        if self.results['link_validation']['broken_internal']:
            section += "### ⚠️ Broken Internal Links\n\n"
            section += "| Source File | Link |\n"
            section += "|--------------|-----|\n"
            for link in self.results['link_validation']['broken_internal'][:20]:
                section += f"| {link['source']} | `{link['url']}` |\n"
            if len(self.results['link_validation']['broken_internal']) > 20:
                section += f"\n*... and {len(self.results['link_validation']['broken_internal']) - 20} more*\n"

        if self.results['link_validation']['broken_external']:
            section += "\n### ⚠️ Broken External Links\n\n"
            section += "| Source File | URL | Error |\n"
            section += "|--------------|-----|-------|\n"
            for link in self.results['link_validation']['broken_external'][:20]:
                section += f"| {link['source']} | `{link['url']}` | {link.get('error', 'N/A')[:50]}... |\n"
            if len(self.results['link_validation']['broken_external']) > 20:
                section += f"\n*... and {len(self.results['link_validation']['broken_external']) - 20} more*\n"

        return section

    def _build_format_issues_section(self) -> str:
        """Build format issues section."""
        if not self.results['format_validation']['format_errors']:
            return "✅ **No format errors found!**"

        section = "### ⚠️ Format Errors\n\n"
        for error in self.results['format_validation']['format_errors'][:10]:
            section += f"- {error}\n"
        if len(self.results['format_validation']['format_errors']) > 10:
            section += f"\n*... and {len(self.results['format_validation']['format_errors']) - 10} more*\n"

        return section

    def _build_stale_docs_section(self) -> str:
        """Build stale documents section."""
        if not self.results['date_currency']['stale_files']:
            return "✅ **No stale documents found!**"

        section = "### ⚠️ Stale Documents (>90 days old)\n\n"
        section += "| File | Last Updated | Days Old |\n"
        section += "|------|--------------|----------|\n"
        for doc in self.results['date_currency']['stale_files'][:20]:
            section += f"| {doc['file']} | {doc['last_updated']} | {doc['days_old']} |\n"
        if len(self.results['date_currency']['stale_files']) > 20:
            section += f"\n*... and {len(self.results['date_currency']['stale_files']) - 20} more*\n"

        return section

    def _build_cross_ref_issues_section(self) -> str:
        """Build cross-reference issues section."""
        if not self.results['cross_references']['broken_references'] and not self.results['cross_references']['orphaned_files']:
            return "✅ **No cross-reference issues found!**"

        section = ""
        if self.results['cross_references']['broken_references']:
            section += "### ⚠️ Broken References\n\n"
            section += "| Source | Target |\n"
            section += "|--------|--------|\n"
            for ref in self.results['cross_references']['broken_references'][:20]:
                section += f"| {ref['source']} | `{ref['target']}` |\n"
            if len(self.results['cross_references']['broken_references']) > 20:
                section += f"\n*... and {len(self.results['cross_references']['broken_references']) - 20} more*\n"

        if self.results['cross_references']['orphaned_files']:
            section += "\n### ⚠️ Orphaned Files (No Incoming Links)\n\n"
            for orphan in self.results['cross_references']['orphaned_files'][:20]:
                section += f"- {orphan}\n"
            if len(self.results['cross_references']['orphaned_files']) > 20:
                section += f"\n*... and {len(self.results['cross_references']['orphaned_files']) - 20} more*\n"

        return section

    def _build_trends_section(self, current_score: float) -> str:
        """Build trends section from history."""
        if not self.history.get('runs'):
            return "📊 **No historical data yet. Trends will appear after multiple runs.**"

        runs = self.history['runs']
        if len(runs) < 2:
            return "📊 **Insufficient data for trends. Need at least 2 runs.**"

        # Get previous score
        prev_score = runs[-2].get('health_score', 0)
        score_change = current_score - prev_score

        # Count issues over time
        prev_broken_links = (
            len(runs[-2].get('link_validation', {}).get('broken_internal', [])) +
            len(runs[-2].get('link_validation', {}).get('broken_external', []))
        )
        curr_broken_links = (
            len(self.results['link_validation']['broken_internal']) +
            len(self.results['link_validation']['broken_external'])
        )

        section = f"""### Health Score Trend

- **Current Score**: {current_score}%
- **Previous Score**: {prev_score}%
- **Change**: {'+' if score_change >= 0 else ''}{score_change:.1f}% {'📈' if score_change > 0 else '📉' if score_change < 0 else '➡️'}

### Broken Links Trend

- **Current**: {curr_broken_links} broken links
- **Previous**: {prev_broken_links} broken links
- **Change**: {'+' if curr_broken_links >= prev_broken_links else ''}{curr_broken_links - prev_broken_links} {'📈' if curr_broken_links > prev_broken_links else '📉' if curr_broken_links < prev_broken_links else '➡️'}

### Historical Summary

- **Total Runs**: {len(runs)}
- **Average Score**: {sum(r.get('health_score', 0) for r in runs) / len(runs):.1f}%
- **Best Score**: {max(r.get('health_score', 0) for r in runs):.1f}%
- **Worst Score**: {min(r.get('health_score', 0) for r in runs):.1f}%
"""
        return section

    def run(self, output_path: Optional[Path] = None) -> bool:
        """Run the complete analysis."""
        logger.info("Starting documentation health analysis...")

        # Run all checks
        self.validate_links()
        logger.info(f"Link validation complete: {self.results['link_validation']['total_links']} links checked")

        self.validate_format()
        logger.info("Format validation complete")

        self.check_date_currency()
        logger.info("Date currency check complete")

        self.validate_cross_references()
        logger.info("Cross-reference validation complete")

        # Calculate health score
        health_score = self.calculate_health_score()
        self.results['health_score'] = health_score
        logger.info(f"Health score: {health_score}%")

        # Save history
        self._save_history()

        # Generate report
        report = self.generate_report(health_score)

        # Write output
        if output_path is None:
            output_path = self.docs_path / 'DOCUMENTATION_HEALTH_REPORT.md'

        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, 'w') as f:
            f.write(report)

        logger.info(f"Report written to: {output_path}")
        return True


def load_config(config_path: Optional[Path] = None) -> Dict:
    """Load configuration from file or use defaults."""
    if config_path is None:
        config_path = project_root / 'scripts' / 'docs_health_config.json'

    default_config = {
        'stale_threshold_days': 90,
        'output_path': 'docs/DOCUMENTATION_HEALTH_REPORT.md'
    }

    if config_path.exists():
        try:
            with open(config_path, 'r') as f:
                user_config = json.load(f)
                default_config.update(user_config)
        except json.JSONDecodeError as e:
            logger.warning(f"Error loading config: {e}, using defaults")
    else:
        logger.info(f"Config file not found: {config_path}, using defaults")

    return default_config


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description='Automated Documentation Health Analysis')
    parser.add_argument('--config', type=Path, help='Path to config file')
    parser.add_argument('--output', type=Path, help='Output path for health report')
    args = parser.parse_args()

    config = load_config(args.config)
    analyzer = DocumentationHealthAnalyzer(config)

    try:
        success = analyzer.run(args.output)
        sys.exit(0 if success else 1)
    except Exception as e:
        logger.error(f"Error running analysis: {e}", exc_info=True)
        sys.exit(1)


if __name__ == '__main__':
    main()
