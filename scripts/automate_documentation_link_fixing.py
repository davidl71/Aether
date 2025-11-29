#!/usr/bin/env python3
"""
Automated Documentation Link Fixing Tool

This script combines both link fixing approaches:
1. Path-based fixing (from fix_documentation_links.py)
2. Name-based fixing (from fix_remaining_doc_links.py)

Designed to be integrated with Exarp automation tools.
"""

import json
import re
import sys
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from datetime import datetime

class DocumentationLinkFixer:
    """Unified documentation link fixing tool"""
    
    def __init__(self, docs_dir: Path, dry_run: bool = True):
        self.docs_dir = docs_dir
        self.dry_run = dry_run
        self.fixes = []
        self.files_modified = []
        self.stats = {
            'total_broken': 0,
            'fixed_path_based': 0,
            'fixed_name_based': 0,
            'unfixable': 0,
            'files_processed': 0,
            'files_modified': 0
        }
    
    def find_alternative_path(self, link_path: str, base_file: Path) -> Optional[str]:
        """Try to find alternative paths for broken links (path-based approach)"""
        base_dir = base_file.parent
        link_name = Path(link_path).name
        
        # Try exact match first
        if (base_dir / link_path).exists():
            return link_path
        
        # Try common alternatives
        alternatives = [
            link_path.lower(),
            link_path.upper(),
            link_path.replace('_', '-'),
            link_path.replace('-', '_'),
            link_name.lower(),
            link_name.upper(),
        ]
        
        # Search in various locations
        search_dirs = [
            base_dir,
            self.docs_dir,
            self.docs_dir / 'research',
            self.docs_dir / 'research' / 'architecture',
            self.docs_dir / 'research' / 'integration',
            self.docs_dir / 'research' / 'external',
            self.docs_dir / 'platform',
            self.docs_dir / 'strategies',
            self.docs_dir / 'archive',
        ]
        
        for alt in alternatives:
            for search_dir in search_dirs:
                target = search_dir / alt
                if target.exists():
                    try:
                        rel_from_docs = target.relative_to(self.docs_dir)
                        base_from_docs = base_dir.relative_to(self.docs_dir)
                        
                        if base_from_docs == Path('.'):
                            return str(rel_from_docs)
                        else:
                            levels_up = len(base_from_docs.parts)
                            rel_path = Path('../' * levels_up) / rel_from_docs
                            return str(rel_path)
                    except ValueError:
                        rel_from_docs = target.relative_to(self.docs_dir)
                        return str(rel_from_docs)
        
        return None
    
    def find_file_by_name(self, target_name: str, base_file: Path) -> Optional[str]:
        """Find a file by name (name-based approach)"""
        base_dir = base_file.parent
        target_stem = Path(target_name).stem.lower()
        
        candidates = []
        for md_file in self.docs_dir.rglob("*.md"):
            if md_file.stem.lower() == target_stem or md_file.name.lower() == target_name.lower():
                try:
                    rel_path = md_file.relative_to(base_dir)
                    candidates.append((md_file, str(rel_path)))
                except ValueError:
                    rel_path = md_file.relative_to(self.docs_dir)
                    base_parts = base_dir.relative_to(self.docs_dir).parts
                    if base_parts == ():
                        candidates.append((md_file, str(rel_path)))
                    else:
                        up_levels = len(base_parts)
                        rel_path_str = '../' * up_levels + str(rel_path)
                        candidates.append((md_file, rel_path_str))
        
        if candidates:
            candidates.sort(key=lambda x: (
                x[0].parent != base_dir,
                len(Path(x[1]).parts)
            ))
            return candidates[0][1]
        
        return None
    
    def fix_links_in_file(self, md_file: Path) -> List[Dict]:
        """Fix broken links in a markdown file using both approaches"""
        fixes = []
        
        try:
            content = md_file.read_text(encoding='utf-8')
            lines = content.split('\n')
            
            for i, line in enumerate(lines):
                matches = list(re.finditer(r'\[([^\]]+)\]\(([^)]+)\)', line))
                if not matches:
                    continue
                
                for match in matches:
                    link_text, link_path = match.groups()
                    
                    # Skip external links and anchors
                    if link_path.startswith('http') or link_path.startswith('#'):
                        continue
                    
                    # Resolve relative path
                    if link_path.startswith('./') or not link_path.startswith('/'):
                        base_dir = md_file.parent
                        target_path = (base_dir / link_path).resolve()
                        
                        if not target_path.exists():
                            self.stats['total_broken'] += 1
                            
                            # Try path-based fixing first
                            alt_path = self.find_alternative_path(link_path, md_file)
                            method = 'path_based'
                            
                            # If path-based fails, try name-based
                            if not alt_path:
                                link_name = Path(link_path).name
                                alt_path = self.find_file_by_name(link_name, md_file)
                                method = 'name_based'
                            
                            if alt_path:
                                old_link = f'[{link_text}]({link_path})'
                                new_link = f'[{link_text}]({alt_path})'
                                lines[i] = lines[i].replace(old_link, new_link)
                                
                                fix_info = {
                                    'line': i + 1,
                                    'old': link_path,
                                    'new': alt_path,
                                    'method': method
                                }
                                fixes.append(fix_info)
                                
                                if method == 'path_based':
                                    self.stats['fixed_path_based'] += 1
                                else:
                                    self.stats['fixed_name_based'] += 1
                            else:
                                self.stats['unfixable'] += 1
            
            if fixes and not self.dry_run:
                md_file.write_text('\n'.join(lines), encoding='utf-8')
                self.files_modified.append(str(md_file.relative_to(self.docs_dir)))
            
            return fixes
        
        except Exception as e:
            print(f"Error processing {md_file}: {e}", file=sys.stderr)
            return []
    
    def run(self) -> Dict:
        """Run the link fixing process"""
        print(f"{'DRY RUN MODE' if self.dry_run else 'FIXING MODE'} - Processing documentation links...\n")
        
        for md_file in sorted(self.docs_dir.rglob("*.md")):
            self.stats['files_processed'] += 1
            fixes = self.fix_links_in_file(md_file)
            
            if fixes:
                self.stats['files_modified'] += 1
                rel_path = md_file.relative_to(self.docs_dir)
                print(f"{rel_path}: {len(fixes)} fixes")
                for fix in fixes[:3]:
                    method_label = "path" if fix['method'] == 'path_based' else "name"
                    print(f"  Line {fix['line']}: {fix['old']} -> {fix['new']} ({method_label})")
                if len(fixes) > 3:
                    print(f"  ... and {len(fixes) - 3} more")
                self.fixes.extend(fixes)
        
        return self.generate_report()
    
    def generate_report(self) -> Dict:
        """Generate a summary report"""
        total_fixed = self.stats['fixed_path_based'] + self.stats['fixed_name_based']
        
        report = {
            'timestamp': datetime.now().isoformat(),
            'dry_run': self.dry_run,
            'stats': self.stats,
            'total_fixed': total_fixed,
            'fix_rate': f"{(total_fixed / self.stats['total_broken'] * 100):.1f}%" if self.stats['total_broken'] > 0 else "0%",
            'files_modified': self.files_modified,
            'status': 'success' if total_fixed > 0 else 'no_fixes_needed'
        }
        
        print(f"\n{'Would fix' if self.dry_run else 'Fixed'} {total_fixed} broken links in {self.stats['files_modified']} files")
        print(f"  Path-based fixes: {self.stats['fixed_path_based']}")
        print(f"  Name-based fixes: {self.stats['fixed_name_based']}")
        print(f"  Unfixable: {self.stats['unfixable']}")
        
        return report


def main():
    """Main entry point"""
    import argparse
    
    parser = argparse.ArgumentParser(description='Automated documentation link fixing')
    parser.add_argument('--dry-run', action='store_true', default=True,
                       help='Dry run mode (default: True)')
    parser.add_argument('--apply', action='store_true',
                       help='Apply fixes (overrides --dry-run)')
    parser.add_argument('--docs-dir', type=str, default='docs',
                       help='Documentation directory (default: docs)')
    parser.add_argument('--output', type=str,
                       help='Output JSON report file')
    
    args = parser.parse_args()
    
    dry_run = not args.apply
    docs_dir = Path(args.docs_dir)
    
    if not docs_dir.exists():
        print(f"Error: Documentation directory '{docs_dir}' not found", file=sys.stderr)
        sys.exit(1)
    
    fixer = DocumentationLinkFixer(docs_dir, dry_run=dry_run)
    report = fixer.run()
    
    if args.output:
        with open(args.output, 'w') as f:
            json.dump(report, f, indent=2)
        print(f"\nReport saved to {args.output}")
    
    if dry_run and report['total_fixed'] > 0:
        print("\nRun with --apply to apply fixes")
    
    sys.exit(0 if report['status'] == 'success' else 1)


if __name__ == '__main__':
    main()
