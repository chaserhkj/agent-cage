set -x
REPO_PATH="${ISOLATED_REPO_PATH:-./.agent-cage-repo}"
REMOTE_NAME="${ISOLATED_REPO_REMOTE:-agent-cage}"
rm -rf "$REPO_PATH"
git remote remove "$REMOTE_NAME"
