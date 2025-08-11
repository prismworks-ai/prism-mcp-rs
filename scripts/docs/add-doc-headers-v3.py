#!/usr/bin/env python3
"""
improved documentation header system v3 with improved messaging.
Adds clear manual/auto-generated labels, 2-click issue reporting, and contributor encouragement.
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
REPO_URL = "https://github.com/prismworks-ai/mcp-protocol-sdk"  # Updated to correct org
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
    body = f"""<!-- Thank you for helping us improve! Your report helps maintain our high documentation standards. -->

### 📍 Document Details
- **File:** `{filepath}`
- **Type:** {"Manually Written" if doc_type == "manual" else "Auto-Generated"}
- **URL:** [{filepath}]({REPO_URL}/blob/main/{filepath})

### Bug: Issue Description
<!-- Please describe what's wrong with the documentation (required) -->



### Note: Suggested Fix (Optional)
<!-- If you know how to fix it, please share! -->



---
*Thank you for helping us maintain the highest documentation standards! Thanks*
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
    """Generate an improved header with clear labeling, 2-click reporting, and contributor encouragement."""
    
    date = datetime.now().strftime("%Y-%m-%d")
    time = datetime.now().strftime("%H:%M:%S UTC")
    filename = os.path.basename(filepath)
    doc_type = "manual" if is_manual else "generated"
    issue_url = create_2click_issue_url(filepath, doc_type)
    contributing_url = f"{REPO_URL}/blob/main/CONTRIBUTING.md"
    
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
Repository: {REPO_URL}
═══════════════════════════════════════════════════════════════
-->

<div align="center">

### Note: Documentation Type: **Manually Written Guide**

[![2-Click Report →](https://img.shields.io/badge/2--Click%20Report%20→-red?style=for-the-badge)]({issue_url})
[![Become a Contributor](https://img.shields.io/badge/Become%20a%20Contributor-blue?style=for-the-badge)]({contributing_url})

**Thank you for helping us maintain the highest documentation standards!**  
*Found an issue? Your 2-click report helps us improve. Want to do more? Join our contributors!*

</div>

---

<div align="center">
<sub>

**# MCP Protocol SDK** - The de facto industry standard for developing MCP clients and servers in Rust  
*Production-ready • 65%+ test coverage • Full protocol compliance • production-ready error handling*

</sub>
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
Repository: {REPO_URL}
═══════════════════════════════════════════════════════════════
-->

<div align="center">

### 🤖 Documentation Type: **Auto-Generated from Source Code**

[![2-Click Report →](https://img.shields.io/badge/2--Click%20Report%20→-orange?style=for-the-badge)]({issue_url})
[![Become a Contributor](https://img.shields.io/badge/Become%20a%20Contributor-blue?style=for-the-badge)]({contributing_url})

**Thank you for helping us maintain the highest documentation standards!**  
*This documentation is automatically generated. To fix issues, please update the source code documentation.*

</div>

---

<div align="center">
<sub>

**# MCP Protocol SDK** - The de facto industry standard for developing MCP clients and servers in Rust  
*Production-ready • 65%+ test coverage • Full protocol compliance • production-ready error handling*

</sub>
</div>

---

"""
    
    return header

def has_v3_header(content: str) -> bool:
    """Check if the file already has a v3 documentation header."""
    return "2-Click Report →" in content or "Become a Contributor" in content

def has_any_header(content: str) -> bool:
    """Check if the file has any version of documentation header."""
    return ("DOCUMENTATION METADATA" in content or 
            "AUTO-GENERATED DOCUMENTATION" in content or
            "Document Type:" in content or
            "Report Documentation Issue" in content)

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
        "has_v3_header": True,
        "repo_url": REPO_URL
    }
    
    # Save metadata
    with open(metadata_file, 'w') as f:
        json.dump(metadata, f, indent=2)

def process_file(filepath: Path) -> bool:
    """Process a single markdown file with improved v3 headers."""
    try:
        # Read the current content
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Check if already has v3 header
        if has_v3_header(content):
            print(f"  ⏭️  {filepath} - already has v3 header")
            return False
        
        # Remove old header if exists
        if has_any_header(content):
            # Find and remove old header
            lines = content.split('\n')
            new_lines = []
            skip_until_separator = False
            separator_count = 0
            
            for line in lines:
                # Check for header start
                if (line.startswith("<!-- Document Type:") or 
                    line.startswith("<!-- \n") or
                    "DOCUMENTATION METADATA" in line or
                    "AUTO-GENERATED DOCUMENTATION" in line):
                    skip_until_separator = True
                    continue
                    
                if skip_until_separator:
                    # Look for the end of header section (after the sub text)
                    if line.strip() == "---":
                        separator_count += 1
                        if separator_count >= 2:  # Skip both --- separators in v2 headers
                            skip_until_separator = False
                        continue
                    elif line.strip() == "" and separator_count > 0:
                        # Empty line after separator, we're done
                        skip_until_separator = False
                        continue
                    else:
                        continue
                        
                new_lines.append(line)
            
            content = '\n'.join(new_lines).lstrip()
            print(f"  🔄 {filepath} - removed old header")
        
        # Determine if manual or generated
        filename = filepath.name
        is_manual = filename in MANUAL_DOCS
        
        # Generate improved v3 header
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
        print(f"  [x] {filepath} - added v3 {doc_type} header with contributor encouragement")
        return True
        
    except Exception as e:
        print(f"  [!] {filepath} - error: {e}")
        return False

def main():
    """Main function to process all documentation files."""
    print("\n# improved Documentation Header System v3\n")
    print("Features:")
    print("  • Clear manual/auto-generated labels")
    print("  • '2-Click Report →' for easy issue reporting")
    print("  • 'Become a Contributor' call-to-action")
    print("  • Project vision and accomplishments highlighted")
    print("  • Correct repository URL (prismworks-ai)\n")
    
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
    
    print(f"\n* Updated {updated} files with improved v3 headers")
    print(f"📊 Metadata saved to {GENERATED_MARKER_FILE}")
    print("\nNote: Headers now include:")
    print("  • '2-Click Report →' badge for easy issue reporting")
    print("  • 'Become a Contributor' badge to encourage contributions")
    print("  • Thank you message for documentation helpers")
    print("  • Project vision statement and key features")
    print("  • Correct GitHub repository URL")

if __name__ == "__main__":
    main()
