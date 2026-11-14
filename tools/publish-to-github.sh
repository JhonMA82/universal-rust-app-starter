#!/usr/bin/env bash
# Publish this starter as a fresh GitHub repository from a local checkout.
# Uses the GitHub CLI; no token is stored in this repository.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_NAME="${1:-universal-rust-app-starter}"
VISIBILITY="${2:-private}"

if [[ ! "$REPO_NAME" =~ ^[a-zA-Z0-9._-]+$ ]]; then
  printf 'Invalid repository name: %s\n' "$REPO_NAME" >&2
  exit 2
fi
if [[ "$VISIBILITY" != private && "$VISIBILITY" != public ]]; then
  printf 'Visibility must be private or public\n' >&2
  exit 2
fi
if ! command -v gh >/dev/null 2>&1; then
  printf 'GitHub CLI (gh) is required: https://cli.github.com/\n' >&2
  exit 1
fi
if ! command -v git >/dev/null 2>&1; then
  printf 'Git is required.\n' >&2
  exit 1
fi
if ! gh auth status >/dev/null 2>&1; then
  printf 'Sign in with: gh auth login\n' >&2
  exit 1
fi

OWNER="$(gh api user --jq '.login')"
USER_ID="$(gh api user --jq '.id')"

cd "$ROOT"
if [[ ! -d .git ]]; then
  git init -b main
fi
if git remote get-url origin >/dev/null 2>&1; then
  printf 'Repository already has an origin remote. Aborting to avoid overwriting it.\n' >&2
  exit 1
fi

# Only files not excluded by .gitignore are staged.
git add -A
if ! git diff --cached --quiet; then
  if ! git -c user.name="${GIT_AUTHOR_NAME:-$OWNER}" \
     -c user.email="${GIT_AUTHOR_EMAIL:-${USER_ID}+${OWNER}@users.noreply.github.com}" \
     commit -m 'chore: initialize Universal Rust App Starter v1.0.0'; then
    printf 'Could not create initial commit. Configure git user.name and user.email and retry.\n' >&2
    exit 1
  fi
fi
if ! git rev-parse HEAD >/dev/null 2>&1; then
  printf 'No commit exists to publish.\n' >&2
  exit 1
fi

# GitHub CLI creates the repository, configures origin, then pushes the current branch.
gh repo create "$REPO_NAME" "--$VISIBILITY" --source . --remote origin --push
if gh api --method PATCH "repos/$OWNER/$REPO_NAME" -F is_template=true >/dev/null; then
  printf '\nGitHub template enabled.\n'
else
  printf '\nWARNING: Could not enable "Template repository" automatically.\n' >&2
  printf 'Enable it in Settings > General > Template repository if you have permission.\n' >&2
fi
printf '\nRepository published: https://github.com/%s/%s\n' "$OWNER" "$REPO_NAME"
