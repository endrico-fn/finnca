#!/usr/bin/env python3
"""Single-command professional release flow (stdlib only, no dependencies).

Bump-in-repo model: versions are raised IN the repo first, then tagged.
Tag format: vMAJOR.MINOR.MAJOR (e.g. v0.1.0). The CI release workflow
verifies the tag matches the manifests and publishes the changelog
section as the GitHub release body (shown in the in-app updater panel).

Usage:
  python3 scripts/release.py [--level patch|minor|major] [--yes] [--check]

  --level   which part to bump from the latest tag (default: patch)
  --yes     actually commit, tag and push (default is dry-run: print plan)
  --check   run quick gates first (pnpm check + cargo check)

Exit codes: 0 ok/dry-run, 1 precondition failed, 2 gate failed.
"""

from __future__ import annotations

import argparse
import datetime
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SEMVER_RE = re.compile(r"^v(\d+)\.(\d+)\.(\d+)$")
CONVENTIONAL_RE = re.compile(
    r"^(feat|fix|docs|refactor|perf|test|chore|ci|build|style)(\([^)]*\))?(!)?:\s*(.+)$",
    re.DOTALL,
)


def run(cmd: list[str], **kwargs) -> str:  # type: ignore[no-untyped-def]
    proc = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, **kwargs)
    if proc.returncode != 0:
        raise RuntimeError(f"command failed: {' '.join(cmd)}\n{proc.stderr.strip()}")
    return proc.stdout


def latest_tag() -> str:
    tags = run(["git", "tag", "--list", "v*.*.*", "--sort=-v:refname"]).split()
    for tag in tags:
        if SEMVER_RE.match(tag):
            return tag
    raise RuntimeError("no previous v*.*.* tag found; create one manually first")


def bump(version: str, level: str) -> str:
    match = SEMVER_RE.match(version)
    if not match:
        raise RuntimeError(f"not a semver tag: {version}")
    major, minor, patch = (int(part) for part in match.groups())
    if level == "major":
        return f"v{major + 1}.0.0"
    if level == "minor":
        return f"v{major}.{minor + 1}.0"
    return f"v{major}.{minor}.{patch + 1}"


def commits_since(tag: str) -> list[tuple[str, str]]:
    out = run(["git", "log", f"{tag}..HEAD", "--pretty=format:%H %s", "--no-merges"])
    entries = []
    for line in out.splitlines():
        line = line.strip()
        if not line:
            continue
        sha, _, subject = line.partition(" ")
        entries.append((sha[:7], subject.strip()))
    return entries


def group_notes(entries: list[tuple[str, str]]) -> dict[str, list[str]]:
    groups: dict[str, list[str]] = {"Added": [], "Fixed": [], "Changed": []}
    for sha, subject in entries:
        match = CONVENTIONAL_RE.match(subject)
        if not match:
            groups["Changed"].append(f"{subject} ({sha})")
            continue
        kind, _, bang, text = match.groups()
        text = " ".join(text.split())
        line = f"{text} ({sha})"
        if bang:
            line = f"⚠ BREAKING: {line}"
        if kind == "feat":
            groups["Added"].append(line)
        elif kind == "fix":
            groups["Fixed"].append(line)
        elif kind in ("docs", "chore", "ci", "test", "style", "build"):
            continue
        else:
            groups["Changed"].append(line)
    return groups


def render_section(version: str, groups: dict[str, list[str]]) -> str:
    today = datetime.date.today().isoformat()
    lines = [f"## [{version[1:]}] - {today}", ""]
    wrote = False
    for heading in ("Added", "Fixed", "Changed"):
        items = groups[heading]
        if not items:
            continue
        wrote = True
        lines.append(f"### {heading}")
        lines.extend(f"- {item}" for item in items)
        lines.append("")
    if not wrote:
        lines.append("### Changed")
        lines.append("- Maintenance release.")
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def replace_first(path: Path, pattern: str, replacement: str) -> None:
    text = path.read_text(encoding="utf-8")
    new_text, count = re.subn(pattern, replacement, text, count=1)
    if count != 1:
        raise RuntimeError(f"version pattern not found exactly once in {path}")
    path.write_text(new_text, encoding="utf-8")


def bump_manifests(old: str, new: str) -> None:
    old_v, new_v = old[1:], new[1:]
    replace_first(
        REPO / "package.json",
        r'"version":\s*"' + re.escape(old_v) + r'"',
        f'"version": "{new_v}"',
    )
    replace_first(
        REPO / "src-tauri" / "tauri.conf.json",
        r'"version":\s*"' + re.escape(old_v) + r'"',
        f'"version": "{new_v}"',
    )
    replace_first(
        REPO / "src-tauri" / "Cargo.toml",
        r'(?m)^version\s*=\s*"' + re.escape(old_v) + r'"',
        f'version = "{new_v}"',
    )
    lock = REPO / "src-tauri" / "Cargo.lock"
    if lock.exists():
        text = lock.read_text(encoding="utf-8")
        new_text, count = re.subn(
            r'(name\s*=\s*"finnca"\nversion\s*=\s*")' + re.escape(old_v) + r'"',
            r"\g<1>" + new_v + '"',
            text,
            count=1,
        )
        if count == 1:
            lock.write_text(new_text, encoding="utf-8")
        else:
            print(
                "WARN: finnca stanza not found in Cargo.lock; CI cargo update will refresh it"
            )


def prepend_changelog(section: str) -> None:
    changelog = REPO / "CHANGELOG.md"
    if not changelog.exists():
        changelog.write_text(
            "# Changelog\n\nAll notable changes to this project follow "
            "[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) "
            "and [Semantic Versioning](https://semver.org/).\n\n## [Unreleased]\n",
            encoding="utf-8",
        )
    text = changelog.read_text(encoding="utf-8")
    marker = "## [Unreleased]"
    if marker not in text:
        raise RuntimeError("CHANGELOG.md has no '## [Unreleased]' section")
    head, _, tail = text.partition(marker)
    tail = tail[len(marker) :]
    changelog.write_text(
        f"{head}{marker}\n\n{section}\n{tail.lstrip()}", encoding="utf-8"
    )


def ensure_clean_tree() -> None:
    status = run(["git", "status", "--porcelain"]).strip()
    if status:
        raise RuntimeError(f"working tree not clean:\n{status}")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Bump version, changelog, commit, tag and push."
    )
    parser.add_argument("--level", choices=["patch", "minor", "major"], default="patch")
    parser.add_argument(
        "--yes", action="store_true", help="execute (default is dry-run)"
    )
    parser.add_argument("--check", action="store_true", help="run quick gates first")
    args = parser.parse_args()

    try:
        ensure_clean_tree()
        old = latest_tag()
        new = bump(old, args.level)
        if run(["git", "tag", "--list", new]).strip():
            raise RuntimeError(f"tag {new} already exists")
        entries = commits_since(old)
        if not entries:
            raise RuntimeError(f"no commits since {old}; nothing to release")
        groups = group_notes(entries)
        section = render_section(new, groups)
    except RuntimeError as err:
        print(f"ERROR: {err}", file=sys.stderr)
        return 1

    print(f"Release {old} -> {new} ({len(entries)} commits)")
    print()
    print(section)
    if not args.yes:
        print("Dry-run only. Re-run with --yes to bump, commit, tag and push.")
        return 0

    if args.check:
        for cmd in (
            ["pnpm", "check"],
            ["cargo", "check", "--manifest-path", "src-tauri/Cargo.toml"],
        ):
            print(f"+ {' '.join(cmd)}")
            proc = subprocess.run(cmd, cwd=REPO)
            if proc.returncode != 0:
                print("ERROR: gate failed, aborting release", file=sys.stderr)
                return 2

    try:
        bump_manifests(old, new)
        prepend_changelog(section)
        run(["git", "add", "-A"])
        run(["git", "commit", "-m", f"chore(release): {new}"])
        run(["git", "tag", "-a", new, "-m", f"Finnca {new}"])
        run(["git", "push", "origin", "main"])
        run(["git", "push", "origin", new])
    except RuntimeError as err:
        print(f"ERROR: {err}", file=sys.stderr)
        print(
            "Release half-applied; inspect git status/log before retrying.",
            file=sys.stderr,
        )
        return 1

    print(f"Released {new}. Watch the Release workflow, then verify latest.json.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
