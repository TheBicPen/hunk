#!/usr/bin/env bash

set -e

if [ $# -ne 1 ]
then
    echo 'Provide a new version number'
    exit 1
fi

sed -i "s/version = .*/version = \"$1\"/" Cargo.toml

# Update the lock file to match the toml file
cargo update --workspace

git add Cargo.lock Cargo.toml
git commit -m "Release $1"
git tag "$1"

echo 'Now run:'
echo ''
echo "git push origin $1"
echo 'cargo publish'
echo ''
