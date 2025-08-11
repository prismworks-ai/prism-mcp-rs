#!/usr/bin/env python3
"""
improved documentation header system with 2-click issue reporting.
Adds clear manual/auto-generated labels and simplified issue reporting.
"""

import os
import sys
import json
import hashlib
from datetime import datetime
from pathlib import Path
import urllib.parse
import re

# Configuration
REPO_URL = "https://github.com/prismworks-ai/mcp-protocol-sdk"
DOCS_DIR = Path("docs")
GENERATED_MARKER_FILE = Path(".local/docs-metadata.json")

# Document classification
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

def get_file_hash(filepath: Path) -> str:
    """Get MD5 hash of file content for tracking changes."""
    with open(filepath, 'rb') as f:
        return hashlib.md5(f.read()).hexdigest()

def create_2click_issue_url(filepath: str, doc_type: str = "manual") -> str:
    """Create a simplified 2-click GitHub issue URL."""
    
    # Pre-fill the issue with all metadata
    title = f"## Documentation Issue: {os.path.basename(filepath)}"
    
    # Determine labels based on doc type
    labels = "documentation,good first issue" if doc_type == "manual" else "documentation,auto-generated"
    
    # Create a simple, focused issue body
    body = f"""<!-- Thank you for reporting this issue! Just fill in the description below and submit. -->

### 📍 Document Details
- **File:** `{filepath}`
- **Type:** {"Manually Written" if doc_type == "manual" else "Auto-Generated"}
- **URL:** [{filepath}]({REPO_URL}/blob/main/{filepath})

### Bug: Issue Description
<!-- Please describe what's wrong with the documentation (required) -->



### Note: Suggested Fix (Optional)
<!-- If you know how to fix it, please share! -->



---
*This issue was created using the 2-click reporting system*"""
    
    # URL encode the parameters
    params = {
        'title': title,
        'labels': labels,
        'body': body
    }
    
    query_string = urllib.parse.urlencode(params)
    return f"{REPO_URL}/issues/new?{query_string}"

def get_improved_header(filepath: str, is_manual: bool = True) -> str:
    """Generate an improved header with clear labeling and 2-click reporting."""
    
    date = datetime.now().strftime("%Y-%m-%d")
    time = datetime.now().strftime("%H:%M:%S UTC")
    filename = os.path.basename(filepath)
    doc_type = "manual" if is_manual else "generated"
    issue_url = create_2click_issue_url(filepath, doc_type)
    
    # Get file hash for tracking
    file_hash = get_file_hash(Path(filepath))[:8]
    
    if is_manual:
        header = f"""<!-- 
═══════════════════════════════════════════════════════════════
## DOCUMENTATION METADATA
═══════════════════════════════════════════════════════════════
Type: User Guide (Manually Written)
Path: {filepath}
Last Updated: {date} {time}
Hash: {file_hash}
═══════════════════════════════════════════════════════════════
-->

<div align="center">

### Note: Documentation Type: **Manually Written Guide**

[![Report Issue](https://img.shields.io/badge/Found%20an%20issue%3F-Report%20it-red?style=for-the-badge)]({issue_url})

*Just click the button above, describe the issue, and submit - that's it!*

</div>

---

"""
    else:
        header = f"""<!-- 
═══════════════════════════════════════════════════════════════
🤖 AUTO-GENERATED DOCUMENTATION
═══════════════════════════════════════════════════════════════
Type: API Reference (Auto-Generated)
Source: Rust source code
Generated: {date} {time}
Generator: scripts/generate-docs.sh
Hash: {file_hash}
═══════════════════════════════════════════════════════════════
-->

<div align="center">

### 🤖 Documentation Type: **Auto-Generated from Source Code**

[![Report Issue](https://img.shields.io/badge/Found%20an%20issue%3F-Report%20it-orange?style=for-the-badge)]({issue_url})

*This documentation is automatically generated. To fix issues, please update the source code documentation.*

</div>

---

"""
    
    return header

def has_v2_header(content: str) -> bool:
    """Check if the file already has a v2 documentation header."""
    return "DOCUMENTATION METADATA" in content or "AUTO-GENERATED DOCUMENTATION" in content

def update_metadata(filepath: Path, is_manual: bool, file_hash: str):
    """Update the metadata tracking file."""
    metadata_file = GENERATED_MARKER_FILE
    metadata_file.parent.mkdir(parents=True, exist_ok=True)
    
    # Load existing metadata
    metadata = {}
    if metadata_file.exists():
        with open(metadata_file, 'r') as f:
            metadata = json.load(f)
    
    # Update metadata
    metadata[str(filepath)] = {
        "type": "manual" if is_manual else "generated",
        "hash": file_hash,
        "last_updated": datetime.now().isoformat(),
        "has_v2_header": True
    }
    
    # Save metadata
    with open(metadata_file, 'w') as f:
        json.dump(metadata, f, indent=2)

def process_file(filepath: Path) -> bool:
    """Process a single markdown file with improved headers."""
    try:
        # Read the current content
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Check if already has v2 header
        if has_v2_header(content):
            print(f"  ⏭️  {filepath} - already has v2 header")
            return False
        
        # Remove old header if exists
        if "<!-- Document Type:" in content:
            # Find and remove old header
            lines = content.split('\n')
            new_lines = []
            skip_until_separator = False
            
            for line in lines:
                if line.startswith("<!-- Document Type:"):
                    skip_until_separator = True
                    continue
                if skip_until_separator:
                    if line.strip() == "---":
                        skip_until_separator = False
                    continue
                new_lines.append(line)
            
            content = '\n'.join(new_lines)
            print(f"  🔄 {filepath} - removed old header")
        
        # Determine if manual or generated
        filename = filepath.name
        is_manual = filename in MANUAL_DOCS
        
        # Generate improved header
        try:
            relative_path = filepath.relative_to(Path.cwd())
        except ValueError:
            relative_path = filepath
        
        header = get_improved_header(str(relative_path), is_manual)
        
        # Add header to content
        new_content = header + content.lstrip()
        
        # Write back
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(new_content)
        
        # Update metadata
        file_hash = get_file_hash(filepath)
        update_metadata(filepath, is_manual, file_hash)
        
        doc_type = "manual" if is_manual else "auto-generated"
        print(f"  [x] {filepath} - added v2 {doc_type} header with 2-click reporting")
        return True
        
    except Exception as e:
        print(f"  [!] {filepath} - error: {e}")
        return False

def main():
    """Main function to process all documentation files."""
    print("\n# improved Documentation Header System v2\n")
    print("Features:")
    print("  • Clear manual/auto-generated labels")
    print("  • 2-click issue reporting (click + describe + submit)")
    print("  • clean badge-based UI")
    print("  • Metadata tracking for changes\n")
    
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
    
    print(f"\n* Updated {updated} files with improved v2 headers")
    print(f"📊 Metadata saved to {GENERATED_MARKER_FILE}")

if __name__ == "__main__":
    main()