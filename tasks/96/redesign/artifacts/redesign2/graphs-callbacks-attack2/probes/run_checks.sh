#!/bin/sh
# Runs today's checker on each probe here; writes ember_check_output.txt. Nothing is built or run.
cd "$(dirname "$0")"
: > ember_check_output.txt
for f in *.em; do
  echo "=== $f" >> ember_check_output.txt
  /c/Users/ism19/Code/ember/target/ci/ember.exe check "$f" >> ember_check_output.txt 2>&1
  echo "exit=$?" >> ember_check_output.txt
done
