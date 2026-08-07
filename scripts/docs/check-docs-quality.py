#!/usr/bin/env python3
"""Validate maintained Markdown documentation.

Checks exact duplicate documents, broken local links, and references to files
that were removed during documentation consolidation.
"""

from __future__ import annotations

import hashlib
import re
import sys
from collections import defaultdict
from pathlib import Path
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[2]
EXCLUDED_PARTS = {".git", "target", ".local"}
REMOVED_PATHS = {
    "docs/DEVELOPMENT.md",
    "docs/index.html",
    "docs/guides/plugin-types.md",
    "docs/ci/LOGGING_GUIDE.md",
    "docs/examples/ai-tool-configs/README.md",
    "scripts/utils/fix-badges.md",
    ".github/badges/doc-examples-badge.md",
}
LINK_RE = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")


def markdown_files() -> list[Path]:
    return sorted(
        path
        for path in ROOT.rglob("*.md")
        if not EXCLUDED_PARTS.intersection(path.relative_to(ROOT).parts)
    )


def normalized_document(path: Path) -> str:
    text = path.read_text(encoding="utf-8")
    text = re.sub(r"<!--.*?-->", "", text, flags=re.DOTALL)
    return re.sub(r"\s+", " ", text).strip().lower()


def duplicate_issues(files: list[Path]) -> list[str]:
    by_digest: dict[str, list[Path]] = defaultdict(list)
    for path in files:
        normalized = normalized_document(path)
        if normalized:
            by_digest[hashlib.sha256(normalized.encode()).hexdigest()].append(path)
    return [
        "exact duplicate documents: "
        + ", ".join(str(path.relative_to(ROOT)) for path in paths)
        for paths in by_digest.values()
        if len(paths) > 1
    ]


def local_link_issues(files: list[Path]) -> list[str]:
    issues = []
    for path in files:
        text = path.read_text(encoding="utf-8")
        for raw_target in LINK_RE.findall(text):
            target = raw_target.strip().split(maxsplit=1)[0].strip("<>")
            if not target or target.startswith(("#", "http://", "https://", "mailto:")):
                continue
            target = unquote(target.split("#", 1)[0].split("?", 1)[0])
            if not target or any(token in target for token in ("${{", "{{", "<")):
                continue
            resolved = (path.parent / target).resolve()
            try:
                resolved.relative_to(ROOT)
            except ValueError:
                issues.append(f"{path.relative_to(ROOT)} links outside repository: {raw_target}")
                continue
            if not resolved.exists():
                issues.append(f"{path.relative_to(ROOT)} has broken link: {raw_target}")
    return issues


def stale_reference_issues(files: list[Path]) -> list[str]:
    issues = []
    for path in files:
        relative = path.relative_to(ROOT)
        text = path.read_text(encoding="utf-8")
        for removed in REMOVED_PATHS:
            if removed in text:
                issues.append(f"{relative} references removed document: {removed}")
    return issues


def main() -> int:
    files = markdown_files()
    issues = (
        duplicate_issues(files)
        + local_link_issues(files)
        + stale_reference_issues(files)
    )
    print(f"Checked {len(files)} Markdown files")
    if issues:
        for issue in issues:
            print(f"ERROR: {issue}")
        return 1
    print("Documentation quality check passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
