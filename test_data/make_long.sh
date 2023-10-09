#!/bin/bash

cd "$(dirname "$0")"
for i in {1..100}
do
    cat 1_color.diff >> long.diff
done
for i in {1..5}
do
    cat long.diff >> longer.diff
done
for i in {1..5}
do
    cat longer.diff >> longest.diff
done
