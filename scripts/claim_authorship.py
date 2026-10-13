#!/usr/bin/env python3
"""Inventory git/GitHub authorship, then rewrite every rewriteable git object
so author and committer are Jojokes (jojo8356).

Usage:
  python3 scripts/claim_authorship.py            # inventory only
  python3 scripts/claim_authorship.py --rewrite  # rewrite current branch vs main
  python3 scripts/claim_authorship.py --rewrite --all-refs  # every local ref
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from collections import Counter

TARGET_NAME = "Jojokes"
TARGET_EMAIL = "83455892+jojo8356@users.noreply.github.com"
TARGET_LOGIN = "jojo8356"
REPO = "jojo8356/schema-pathfinder"
MAIN = "origin/main"

# GitHub event / object kinds we can observe. Only git commits/tags/notes
# can actually have their author rewritten locally.
ACTION_TYPES = [
    ("git.commit", True, "Commit (author + committer)"),
    ("git.merge_commit", True, "Commit de merge"),
    ("git.tag_annotated", True, "Tag annoté (tagger)"),
    ("git.tag_lightweight", True, "Tag léger (pointe sur un commit réécrit)"),
    ("git.branch", True, "Branche (pointeurs, via les commits)"),
    ("git.note", True, "Notes git"),
    ("github.pull_request", False, "Auteur d'une PR (immuable)"),
    ("github.pull_request_review", False, "Review GitHub (immuable)"),
    ("github.issue", False, "Issue (immuable)"),
    ("github.issue_comment", False, "Commentaire (immuable)"),
    ("github.release", False, "Release (publisher immuable sans delete/recreate)"),
    ("github.workflow_run", False, "GitHub Actions run actor (immuable)"),
    ("github.deployment", False, "Deployment (immuable)"),
]


def run(cmd: list[str], check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, text=True, capture_output=True, check=check)


def git(*args: str, check: bool = True) -> str:
    result = run(["git", *args], check=check)
    return (result.stdout or "").strip()


def gh_json(args: list[str]):
    result = run(["gh", *args], check=False)
    if result.returncode != 0:
        return None
    text = result.stdout.strip()
    if not text:
        return None
    return json.loads(text)


def inventory() -> dict:
    authors = Counter()
    committers = Counter()
    merges = 0
    for line in git("log", "--all", "--format=%an <%ae>\t%cn <%ce>\t%P").splitlines():
        if not line.strip():
            continue
        parts = line.split("\t")
        author = parts[0]
        committer = parts[1] if len(parts) > 1 else parts[0]
        parents = parts[2] if len(parts) > 2 else ""
        authors[author] += 1
        committers[committer] += 1
        if parents.count(" ") >= 1 and parents.strip():
            merges += 1

    tags = git("tag", "-l").splitlines() if git("tag", "-l") else []
    branches = git("branch", "-a", "--format=%(refname:short)").splitlines()

    releases = gh_json(["release", "list", "-R", REPO, "--limit", "50", "--json", "tagName,author,createdAt"]) or []
    prs = gh_json(["pr", "list", "-R", REPO, "--state", "all", "--limit", "50", "--json", "number,author,title"]) or []

    return {
        "authors": authors,
        "committers": committers,
        "merges": merges,
        "tags": tags,
        "branches": branches,
        "releases": releases,
        "prs": prs,
    }


def print_inventory(data: dict) -> None:
    print("=== Types d'actions ===")
    for kind, rewriteable, label in ACTION_TYPES:
        flag = "RÉÉCRIT" if rewriteable else "IMMUTABLE"
        print(f"  [{flag:10}] {kind:28} {label}")

    print("\n=== Auteurs des commits (author) ===")
    for who, count in data["authors"].most_common():
        print(f"  {count:5}  {who}")
    print("=== Committers ===")
    for who, count in data["committers"].most_common():
        print(f"  {count:5}  {who}")
    print(f"\nMerges: {data['merges']}")
    print("Tags:", ", ".join(data["tags"]) or "(aucun)")
    print("Branches:")
    for branch in data["branches"]:
        print(f"  {branch}")
    print("\n=== Releases GitHub ===")
    for rel in data["releases"]:
        author = (rel.get("author") or {}).get("login", "?")
        print(f"  {rel.get('tagName')}  publisher={author}")
    print("\n=== Pull requests (auteur immuable) ===")
    for pr in data["prs"][:20]:
        login = (pr.get("author") or {}).get("login", "?")
        print(f"  #{pr.get('number')}  {login}  {pr.get('title')}")

    print(f"\nCible: {TARGET_NAME} <{TARGET_EMAIL}>  (login {TARGET_LOGIN})")


def rewrite(all_refs: bool) -> None:
    env_filter = f"""
export GIT_AUTHOR_NAME="{TARGET_NAME}"
export GIT_AUTHOR_EMAIL="{TARGET_EMAIL}"
export GIT_COMMITTER_NAME="{TARGET_NAME}"
export GIT_COMMITTER_EMAIL="{TARGET_EMAIL}"
"""
    if all_refs:
        rev_list = ["--", "--all"]
    else:
        merge_base = git("merge-base", MAIN, "HEAD", check=False)
        if not merge_base:
            rev_list = ["--", "HEAD"]
        else:
            rev_list = ["--", f"{merge_base}..HEAD"]

    cmd = [
        "git",
        "filter-branch",
        "-f",
        "--env-filter",
        env_filter,
        "--msg-filter",
        r"sed -e '/Co-authored-by: arena-agent/d' -e '/Co-authored-by: Arena Agent/d'",
        *rev_list,
    ]
    print("RUN", " ".join(cmd[:6]), "...")
    result = subprocess.run(cmd)
    if result.returncode != 0:
        sys.exit(result.returncode)
    print("Rewrite done. Force-push the branch if it was already published:")
    print("  git push --force-with-lease origin HEAD")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rewrite", action="store_true")
    parser.add_argument("--all-refs", action="store_true")
    args = parser.parse_args()

    data = inventory()
    print_inventory(data)
    if args.rewrite:
        rewrite(args.all_refs)
        print("\n=== Après réécriture ===")
        print_inventory(inventory())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
