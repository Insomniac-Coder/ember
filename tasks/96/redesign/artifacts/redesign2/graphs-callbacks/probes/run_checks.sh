#!/bin/sh
# Runs `ember check` on every probe here and saves the output (no Ember program is run).
cd "$(dirname "$0")"
out=ember_check_output.txt
: > $out
for f in *.em; do
  echo "=== ember check probes/$f" >> $out
  /c/Users/ism19/Code/ember/target/ci/ember.exe check "$f" >> $out 2>&1
  echo "exit=$?" >> $out
done
