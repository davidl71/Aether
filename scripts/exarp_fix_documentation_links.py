#!/usr/bin/env python3
"""
Exarp-compatible wrapper for documentation link fixing.

This script follows Exarp's script pattern and can be integrated into
Exarp's daily automation or called directly.
"""

import json
import sys
from pathlib import Path

# Add parent directory to path to import our automation script
sys.path.insert(0, str(Path(__file__).parent))

from automate_documentation_link_fixing import DocumentationLinkFixer


def main():
    """Main entry point following Exarp script pattern"""
    import argparse
    
    parser = argparse.ArgumentParser(
        description='Fix broken documentation links (Exarp-compatible)'
    )
    parser.add_argument(
        'project_dir',
        nargs='?',
        default='.',
        help='Project root directory (default: current directory)'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        default=True,
        help='Dry run mode (default: True)'
    )
    parser.add_argument(
        '--apply',
        action='store_true',
        help='Apply fixes (overrides --dry-run)'
    )
    parser.add_argument(
        '--output',
        type=str,
        help='Output JSON report file (optional)'
    )
    parser.add_argument(
        '--json',
        action='store_true',
        help='Output results as JSON'
    )
    
    args = parser.parse_args()
    
    project_dir = Path(args.project_dir).resolve()
    docs_dir = project_dir / 'docs'
    
    if not docs_dir.exists():
        error_msg = {
            'status': 'error',
            'message': f'Documentation directory not found: {docs_dir}',
            'project_dir': str(project_dir)
        }
        if args.json:
            print(json.dumps(error_msg, indent=2))
        else:
            print(f"❌ Error: {error_msg['message']}", file=sys.stderr)
        sys.exit(1)
    
    dry_run = not args.apply
    
    # Run the link fixer
    fixer = DocumentationLinkFixer(docs_dir, dry_run=dry_run)
    report = fixer.run()
    
    # Output results
    if args.json:
        print(json.dumps(report, indent=2))
    elif args.output:
        with open(args.output, 'w') as f:
            json.dump(report, f, indent=2)
        print(f"\n✅ Report saved to {args.output}")
    
    # Exit with appropriate code
    if report['status'] == 'error':
        sys.exit(1)
    elif report['total_fixed'] == 0 and report['stats']['total_broken'] > 0:
        # No fixes applied but broken links exist
        sys.exit(2)
    else:
        sys.exit(0)


if __name__ == '__main__':
    main()
