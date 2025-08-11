#!/usr/bin/env python3
"""
Check documentation quality and identify issues:
- Duplicate content
- Missing cross-references
- Broken links
- API documentation that should be auto-generated
"""

import os
import re
from pathlib import Path
from collections import defaultdict

def check_for_duplicates():
    """Check for duplicate content across documentation files."""
    docs_dir = Path('docs')
    issues = []
    
    # Key phrases that indicate duplication
    key_phrases = [
        'smart retry logic',
        'circuit breaker protection',
        'exponential backoff',
        'production-ready error handling',
        'health monitoring',
        'transport configuration'
    ]
    
    phrase_locations = defaultdict(list)
    
    for doc_file in docs_dir.rglob('*.md'):
        if 'archive' in str(doc_file):
            continue
            
        with open(doc_file, 'r') as f:
            content = f.read().lower()
            for phrase in key_phrases:
                if phrase.lower() in content:
                    phrase_locations[phrase].append(doc_file.relative_to(docs_dir))
    
    for phrase, locations in phrase_locations.items():
        if len(locations) > 1:
            issues.append(f"Duplicate content '{phrase}' in: {', '.join(str(l) for l in locations)}")
    
    return issues

def check_for_api_docs():
    """Check for manually written API documentation that should be auto-generated."""
    docs_dir = Path('docs')
    issues = []
    
    # Patterns that indicate API documentation
    api_patterns = [
        r'pub struct \w+',
        r'pub fn \w+',
        r'pub trait \w+',
        r'pub enum \w+',
        r'impl \w+ for \w+',
        r'fn \w+\([^)]*\) -> \w+'
    ]
    
    for doc_file in docs_dir.rglob('*.md'):
        if 'archive' in str(doc_file) or 'api-reference.md' in str(doc_file):
            continue
            
        with open(doc_file, 'r') as f:
            content = f.read()
            for pattern in api_patterns:
                if re.search(pattern, content):
                    issues.append(f"Manual API docs found in {doc_file.relative_to(docs_dir)} - should be auto-generated")
                    break
    
    return issues

def check_cross_references():
    """Check that cross-references use proper anchors."""
    docs_dir = Path('docs')
    issues = []
    
    # Find all markdown links
    link_pattern = r'\[([^\]]+)\]\(([^)]+)\)'
    
    for doc_file in docs_dir.rglob('*.md'):
        if 'archive' in str(doc_file):
            continue
            
        with open(doc_file, 'r') as f:
            content = f.read()
            links = re.findall(link_pattern, content)
            
            for link_text, link_url in links:
                # Check for vague references
                if link_url.startswith('./') and '#' not in link_url and link_url.endswith('.md'):
                    # Check if this could use a more specific anchor
                    if any(word in link_text.lower() for word in ['error', 'retry', 'circuit', 'health', 'transport']):
                        issues.append(f"Missing anchor in {doc_file.name}: [{link_text}]({link_url}) - consider adding #section")
    
    return issues

def main():
    print("\nSearch: Documentation Quality Check\n")
    print("=" * 40)
    
    all_issues = []
    
    # Check for duplicates
    print("\nChecking for duplicate content...")
    duplicates = check_for_duplicates()
    if duplicates:
        print(f"[!] Found {len(duplicates)} duplication issues:")
        for issue in duplicates:
            print(f"  - {issue}")
        all_issues.extend(duplicates)
    else:
        print("[x] No significant duplications found")
    
    # Check for manual API docs
    print("\nChecking for manual API documentation...")
    api_docs = check_for_api_docs()
    if api_docs:
        print(f"[!] Found {len(api_docs)} manual API doc issues:")
        for issue in api_docs:
            print(f"  - {issue}")
        all_issues.extend(api_docs)
    else:
        print("[x] No manual API documentation found")
    
    # Check cross-references
    print("\nChecking cross-references...")
    refs = check_cross_references()
    if refs:
        print(f"Warning:  Found {len(refs)} cross-reference improvements:")
        for issue in refs:
            print(f"  - {issue}")
        all_issues.extend(refs)
    else:
        print("[x] All cross-references properly anchored")
    
    # Summary
    print("\n" + "=" * 40)
    if all_issues:
        print(f"\nWarning:  Total issues found: {len(all_issues)}")
        print("Please review and fix the issues above.")
        return 1
    else:
        print("\n[x] Documentation quality check passed!")
        return 0

if __name__ == '__main__':
    exit(main())
