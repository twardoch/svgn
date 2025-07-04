#!/usr/bin/env bash
cd $(dirname "$0")/..
echo "original"
cat testdata/complex.svg
echo
echo "--------------------------------"
echo "svgn"
cat testdata/complex.svg | ./target/release/svgn -i - -o -
echo
echo "--------------------------------"
echo "svgo"
cat testdata/complex.svg | npx svgo -i - -o -
echo "--------------------------------"
