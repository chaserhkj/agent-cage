set -e
[ -d ./agent-cage-repo ] && { echo "agent-cage-repo/ already present, skipping preparation"; exit 0; }
echo "preparing isolated git repo at agent-cage-repo/"
set -x
mkdir ./agent-cage-repo
(
    set -ex
    cd ./agent-cage-repo
    git init
    git config receive.denyCurrentBranch updateInstead
)
git remote add agent-cage-repo ./agent-cage-repo
git push agent-cage-repo main