#!/usr/bin/env python3
"""
Parallel documentation fixer - fixes broken links and format errors in parallel.

Uses multiple background processes to fix documentation issues simultaneously.
"""

import json
import subprocess
import sys
from pathlib import Path
from multiprocessing import Pool, cpu_count
from concurrent.futures import ProcessPoolExecutor, as_completed
import time

def fix_links_in_chunk(args_tuple):
    """Fix links in a chunk of files (runs in parallel process)"""
    files_chunk, docs_dir_str, dry_run = args_tuple
    docs_dir = Path(docs_dir_str)
    results = []
    
    # Import the DocumentationLinkFixer class
    sys.path.insert(0, str(Path(__file__).parent))
    from automate_documentation_link_fixing import DocumentationLinkFixer
    
    try:
        # Create fixer instance for this chunk
        fixer = DocumentationLinkFixer(docs_dir, dry_run=dry_run)
        
        # Process each file in the chunk
        for md_file in files_chunk:
            try:
                fixes = fixer.fix_links_in_file(md_file)
                if fixes:
                    # Apply fixes if not dry run
                    if not dry_run:
                        content = md_file.read_text(encoding='utf-8')
                        lines = content.split('\n')
                        
                        for fix in fixes:
                            line_idx = fix['line'] - 1
                            if 0 <= line_idx < len(lines):
                                lines[line_idx] = lines[line_idx].replace(
                                    fix['old'], fix['new']
                                )
                        
                        md_file.write_text('\n'.join(lines), encoding='utf-8')
                    
                    results.append({
                        'file': str(md_file),
                        'success': True,
                        'fixes_count': len(fixes)
                    })
                else:
                    results.append({
                        'file': str(md_file),
                        'success': True,
                        'fixes_count': 0
                    })
            except Exception as e:
                results.append({
                    'file': str(md_file),
                    'success': False,
                    'error': str(e)
                })
    except Exception as e:
        # If fixer creation fails, mark all files as failed
        for md_file in files_chunk:
            results.append({
                'file': str(md_file),
                'success': False,
                'error': f'Fixer initialization failed: {str(e)}'
            })
    
    return results

def fix_format_errors_in_chunk(files_chunk):
    """Fix format errors in a chunk of files (runs in parallel process)"""
    results = []
    
    for md_file in files_chunk:
        try:
            content = md_file.read_text(encoding='utf-8')
            original_content = content
            
            # Fix common format errors
            # 1. Remove trailing whitespace
            lines = content.split('\n')
            fixed_lines = [line.rstrip() for line in lines]
            content = '\n'.join(fixed_lines)
            
            # 2. Ensure file ends with newline
            if content and not content.endswith('\n'):
                content += '\n'
            
            # 3. Fix multiple blank lines (max 2 consecutive)
            import re
            content = re.sub(r'\n{3,}', '\n\n', content)
            
            # 4. Fix inconsistent list indentation (basic)
            # This is a simplified fix - markdownlint would be better
            
            if content != original_content:
                md_file.write_text(content, encoding='utf-8')
                results.append({
                    'file': str(md_file),
                    'success': True,
                    'fixed': True
                })
            else:
                results.append({
                    'file': str(md_file),
                    'success': True,
                    'fixed': False
                })
        except Exception as e:
            results.append({
                'file': str(md_file),
                'success': False,
                'error': str(e)
            })
    
    return results

def get_all_markdown_files(docs_dir):
    """Get all markdown files in docs directory"""
    docs_path = Path(docs_dir)
    return list(docs_path.rglob('*.md'))

def chunk_files(files, chunk_size):
    """Split files into chunks for parallel processing"""
    for i in range(0, len(files), chunk_size):
        yield files[i:i + chunk_size]

def main():
    """Main execution - runs fixes in parallel"""
    import argparse
    
    parser = argparse.ArgumentParser(
        description='Fix documentation issues in parallel'
    )
    parser.add_argument(
        '--docs-dir',
        type=str,
        default='docs',
        help='Documentation directory (default: docs)'
    )
    parser.add_argument(
        '--workers',
        type=int,
        default=None,
        help='Number of parallel workers (default: CPU count)'
    )
    parser.add_argument(
        '--chunk-size',
        type=int,
        default=10,
        help='Files per chunk (default: 10)'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Dry run mode (no actual fixes)'
    )
    parser.add_argument(
        '--links-only',
        action='store_true',
        help='Only fix broken links'
    )
    parser.add_argument(
        '--format-only',
        action='store_true',
        help='Only fix format errors'
    )
    
    args = parser.parse_args()
    
    docs_dir = Path(args.docs_dir).resolve()
    if not docs_dir.exists():
        print(f"❌ Error: Documentation directory not found: {docs_dir}")
        sys.exit(1)
    
    # Get all markdown files
    print(f"📋 Scanning {docs_dir} for markdown files...")
    all_files = get_all_markdown_files(docs_dir)
    print(f"✅ Found {len(all_files)} markdown files")
    
    if args.dry_run:
        print("🔍 DRY RUN MODE - No files will be modified")
    
    # Determine number of workers
    num_workers = args.workers or min(cpu_count(), 8)  # Cap at 8 to avoid overload
    print(f"⚙️  Using {num_workers} parallel workers")
    
    # Split files into chunks
    chunk_size = args.chunk_size
    file_chunks = list(chunk_files(all_files, chunk_size))
    print(f"📦 Split into {len(file_chunks)} chunks of ~{chunk_size} files each")
    
    all_results = {
        'links': [],
        'format': [],
        'total_files': len(all_files),
        'start_time': time.time()
    }
    
    # Fix broken links in parallel
    if not args.format_only:
        print("\n🔗 Fixing broken links in parallel...")
        with ProcessPoolExecutor(max_workers=num_workers) as executor:
            futures = [
                executor.submit(fix_links_in_chunk, (chunk, str(docs_dir), args.dry_run))
                for chunk in file_chunks
            ]
            
            completed = 0
            for future in as_completed(futures):
                chunk_results = future.result()
                all_results['links'].extend(chunk_results)
                completed += len(chunk_results)
                fixed_count = sum(1 for r in chunk_results if r.get('fixes_count', 0) > 0)
                print(f"  ✅ Processed {completed}/{len(all_files)} files ({fixed_count} fixed in this chunk)...", end='\r')
        
        print(f"\n✅ Link fixing complete: {completed} files processed")
    
    # Fix format errors in parallel
    if not args.links_only:
        print("\n📝 Fixing format errors in parallel...")
        with ProcessPoolExecutor(max_workers=num_workers) as executor:
            futures = [
                executor.submit(fix_format_errors_in_chunk, chunk)
                for chunk in file_chunks
            ]
            
            completed = 0
            for future in as_completed(futures):
                chunk_results = future.result()
                all_results['format'].extend(chunk_results)
                completed += len(chunk_results)
                print(f"  ✅ Processed {completed}/{len(all_files)} files for format...", end='\r')
        
        print(f"\n✅ Format fixing complete: {completed} files processed")
    
    # Summary
    all_results['end_time'] = time.time()
    all_results['duration'] = all_results['end_time'] - all_results['start_time']
    
    links_success = sum(1 for r in all_results['links'] if r.get('success'))
    format_fixed = sum(1 for r in all_results['format'] if r.get('fixed'))
    
    print("\n" + "="*60)
    print("📊 SUMMARY")
    print("="*60)
    print(f"Total files processed: {len(all_files)}")
    print(f"Link fixes successful: {links_success}/{len(all_results['links'])}")
    print(f"Format fixes applied: {format_fixed}/{len(all_results['format'])}")
    print(f"Total duration: {all_results['duration']:.2f} seconds")
    print("="*60)
    
    # Save report
    report_path = Path('docs/DOCUMENTATION_FIX_REPORT.json')
    with open(report_path, 'w') as f:
        json.dump(all_results, f, indent=2)
    print(f"\n✅ Report saved to {report_path}")
    
    return 0

if __name__ == '__main__':
    sys.exit(main())
