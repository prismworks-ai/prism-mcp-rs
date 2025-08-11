#!/usr/bin/env python3
"""
Add documentation headers to all markdown files in the docs directory.
This script adds issue reporting links and marks whether docs are manual or auto-generated.
"""

import os
import sys
from datetime import datetime
from pathlib import Path
import urllib.parse

# Configuration
REPO_URL = "https://github.com/prismworks-ai/mcp-protocol-sdk"
DOCS_DIR = Path("docs")
MANUAL_DOCS = [
    "README.md",
    "getting-started.md", 
    "transports.md",
    "error-handling.md",
    "health-monitoring.md",
    "production-readiness.md",
    "SECURITY.md"
]

GENERATED_DOCS = [
    "api-reference.md"
]

def create_issue_url(filepath: str, section: str = "") -> str:
    """Create a GitHub issue URL with pre-filled information."""
    title = f"[Docs] Issue in {filepath}"
    
    # Create the issue body
    body = f"""## Documentation Issue

**Page:** `{filepath}`
**Section:** {section if section else '[Please specify the section]'}

**Issue Description:**
[Please describe the documentation issue you found]

**Expected Content:**
[What should the documentation say?]

**Additional Context:**
[Any additional information that might be helpful]
"""
    
    # URL encode the parameters
    params = {
        'template': 'doc_issue.md',
        'labels': 'documentation',
        'title': title,
        'body': body
    }
    
    query_string = urllib.parse.urlencode(params)
    return f"{REPO_URL}/issues/new?{query_string}"

def get_header(filepath: str, is_manual: bool = True) -> str:
    """Generate the appropriate header for a documentation file."""
    date = datetime.now().strftime("%Y-%m-%d")
    filename = os.path.basename(filepath)
    issue_url = create_issue_url(filepath)
    
    if is_manual:
        header = f"""<!-- Document Type: User Guide -->
<!-- Source: Manually Written -->
<!-- Last Updated: {date} -->

> Note: **This is a manually written guide** | [Report Documentation Issue]({issue_url})

---

"""
    else:
        header = f"""<!-- Document Type: API Reference -->
<!-- Source: Auto-Generated from Source Code -->
<!-- Generation Date: {date} -->
<!-- Generator: scripts/generate-docs.sh -->

> 🤖 **This documentation is auto-generated** from source code | [Report Documentation Issue]({issue_url})

---

"""
    
    return header

def has_header(content: str) -> bool:
    """Check if the file already has a documentation header."""
    return "<!-- Document Type:" in content or "Report Documentation Issue" in content

def process_file(filepath: Path) -> bool:
    """Process a single markdown file."""
    try:
        # Read the current content
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Skip if already has header
        if has_header(content):
            print(f"  ⏭️  {filepath} - already has header")
            return False
        
        # Determine if manual or generated
        filename = filepath.name
        is_manual = filename in MANUAL_DOCS
        is_generated = filename in GENERATED_DOCS
        
        if not is_manual and not is_generated:
            # Default to manual for unknown files
            is_manual = True
        
        # Generate header
        try:
            relative_path = filepath.relative_to(Path.cwd())
        except ValueError:
            # If relative_to fails, use the path as is
            relative_path = filepath
        header = get_header(str(relative_path), is_manual)
        
        # Add header to content
        new_content = header + content
        
        # Write back
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(new_content)
        
        doc_type = "manual" if is_manual else "auto-generated"
        print(f"  [x] {filepath} - added {doc_type} header")
        return True
        
    except Exception as e:
        print(f"  [!] {filepath} - error: {e}")
        return False

def main():
    """Main function to process all documentation files."""
    print("Adding documentation headers...\n")
    
    if not DOCS_DIR.exists():
        print(f"Error: {DOCS_DIR} directory not found")
        sys.exit(1)
    
    # Find all markdown files
    md_files = list(DOCS_DIR.glob("**/*.md"))
    
    if not md_files:
        print("No markdown files found in docs directory")
        return
    
    print(f"Found {len(md_files)} markdown files\n")
    
    updated = 0
    for md_file in md_files:
        # Skip files in archive or backup directories
        if 'archive' in str(md_file) or 'backup' in str(md_file):
            continue
            
        if process_file(md_file):
            updated += 1
    
    print(f"\n* Updated {updated} files with documentation headers")

if __name__ == "__main__":
    main()