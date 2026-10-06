#!/usr/bin/env bash
# The README's full benchmark run, from Git Bash on the Windows laptop: the Windows set (MSVC,
# then clang, on the performance cores), then a Linux copy of the compiler in WSL and the gcc
# set. Every step is on mains power or the run stops. Logs go to build/bench/logs/; the WSL copy
# and its builds are deleted at the end. Then, if the run is the README's: copy the logs to
# bench/results/ and run bench/update_readme.py on them (see bench/README.md).
set -u
cd "$(dirname "$0")/.."
stamp=$(date +%Y-%m-%d)
logs=build/bench/logs
mkdir -p "$logs"
mains() { python -c "import sys; sys.path.insert(0, 'bench'); from power_guard import require_mains; require_mains('$1')"; }
wsl_run() { MSYS_NO_PATHCONV=1 wsl -e bash -lc "$1"; }
echo "windows start $(date +%T)"
python bench/run_windows.py "$logs/$stamp-windows.log" 11 || { echo "windows run failed"; exit 1; }
mains "wsl tree" || exit 1
wsl_run "bash /mnt/c/Users/ism19/Code/ember/bench/gcc/make_tree.sh" > "$logs/$stamp-wsl-tree.log" 2>&1 \
  || { echo "wsl tree failed"; tail -20 "$logs/$stamp-wsl-tree.log"; exit 1; }
mains "gcc" || exit 1
echo "gcc start $(date +%T)"
wsl_run "python3 /mnt/c/Users/ism19/Code/ember/bench/gcc/run_gcc.py 11" > "$logs/$stamp-gcc.log" 2>&1 \
  || { echo "gcc run failed"; tail -20 "$logs/$stamp-gcc.log"; exit 1; }
mains "the end" || exit 1
wsl_run 'rm -rf ~/ember-bench-tree ~/ember-bench-out'
echo "done $(date +%T): $logs/$stamp-windows.log, $logs/$stamp-gcc.log"
