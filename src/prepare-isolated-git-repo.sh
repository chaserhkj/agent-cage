set -e
[ -d ./agent-cage-repo ] && echo "agent-cage-repo/ already present, skipping preparation"
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
git branch agent-cage
git push -u agent-cage-repo agent-cage:main