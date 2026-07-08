set -x
REPO_PATH="${ISOLATED_REPO_PATH:-./.agent-cage-repo}"
REMOTE_NAME="${ISOLATED_REPO_REMOTE:-agent-cage}"

GIT_DIR=$(git rev-parse --git-dir)
EXCLUDE_FILE="$GIT_DIR/info/exclude"
[ -f "$EXCLUDE_FILE" ] && sed -i '/# agent-cage-isolated-repo/d' "$EXCLUDE_FILE"

rm -rf "$REPO_PATH"
git remote remove "$REMOTE_NAME"
