set -e
REPO_PATH="${ISOLATED_REPO_PATH:-./.agent-cage-repo}"
REMOTE_NAME="${ISOLATED_REPO_REMOTE:-agent-cage}"
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
