#!/usr/bin/env bash

set -e

if [ $# -ne 1 ]
then
    echo 'Provide a new version number'
    exit 1
fi

git tag "$1"
sed -i "s/version = .*/version = \"$1\"/" Cargo.toml

echo 'Now run:'
echo ''
echo "git push origin $1"
echo 'cargo publish'
echo ''
