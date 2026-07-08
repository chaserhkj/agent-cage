set -x
REPO_PATH="${ISOLATED_REPO_PATH:-./.agent-cage-repo}"
REMOTE_NAME="${ISOLATED_REPO_REMOTE:-agent-cage}"

GIT_DIR=$(git rev-parse --git-dir)
EXCLUDE_FILE="$GIT_DIR/info/exclude"
[ -f "$EXCLUDE_FILE" ] && grep -v '# agent-cage-isolated-repo' "$EXCLUDE_FILE" > "$EXCLUDE_FILE.tmp" 2>/dev/null && mv "$EXCLUDE_FILE.tmp" "$EXCLUDE_FILE" || rm -f "$EXCLUDE_FILE.tmp"

rm -rf "$REPO_PATH"
git remote remove "$REMOTE_NAME"
