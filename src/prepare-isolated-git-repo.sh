set -ex
REPO_PATH="${ISOLATED_REPO_PATH:-./.agent-cage-repo}"
REMOTE_NAME="${ISOLATED_REPO_REMOTE:-agent-cage}"

# Detect legacy agent-cage-repo directory (pre-rename) and bail with migration instructions
LEGACY_DIR="./agent-cage-repo"
if [ -d "$LEGACY_DIR" ]; then
    echo "ERROR: Legacy isolated repo directory '$LEGACY_DIR' detected." >&2
    echo "" >&2
    echo "The default directory was renamed from 'agent-cage-repo/' to '.agent-cage-repo/'." >&2
    echo "To migrate manually:" >&2
    echo "  1. mv ./agent-cage-repo ./.agent-cage-repo" >&2
    echo "  2. git remote rm agent-cage-repo" >&2
    echo "  3. git remote add agent-cage ./.agent-cage-repo" >&2
    echo "  4. git push agent-cage main" >&2
    echo "" >&2
    echo "Or use the --isolated-repo-path and --isolated-repo-remote flags to keep using the old names." >&2
    exit 1
fi

[ -d "$REPO_PATH" ] && { echo "$REPO_PATH/ already present, skipping preparation"; exit 0; }
echo "preparing isolated git repo at $REPO_PATH/"
mkdir "$REPO_PATH"
(
    set -ex
    cd "$REPO_PATH"
    git init
    git config receive.denyCurrentBranch updateInstead
)
git remote add "$REMOTE_NAME" "$REPO_PATH"
git push "$REMOTE_NAME" main

GIT_DIR=$(git rev-parse --git-dir)
EXCLUDE_FILE="$GIT_DIR/info/exclude"
mkdir -p "$(dirname "$EXCLUDE_FILE")"
touch "$EXCLUDE_FILE"
echo "$REPO_PATH/ # agent-cage-isolated-repo" >> "$EXCLUDE_FILE"
