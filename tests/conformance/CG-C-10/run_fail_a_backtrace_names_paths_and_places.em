#$ test: run-fail
#$ rules: CG-C-10, RT-4
#$ profiles: debug
#$ stdout: 8
#$ panics: index 5 is out of bounds for a length of 2
#$ stderr: panic at support/geometry.em:8:16: index 5 is out of bounds for a length of 2
#$ stderr: backtrace:
#$ stderr: 0: support.geometry.Grid.at at support/geometry.em:8:16
#$ stderr: 1: support.geometry.probe at support/geometry.em:11:17
#$ stderr: 2: outer at run_fail_a_backtrace_names_paths_and_places.em:24:30
#$ stderr: 3: main at run_fail_a_backtrace_names_paths_and_places.em:32:22
# `[CG-C-10]`: a stack trace prints each frame by its Ember path, a method's type and name
# included (`support.geometry.Grid.at`), and where it is, `file:line:col`: the innermost frame at the
# panic, each other at its call of the frame below it (the column of the call's last argument, of
# the call itself when it has none). `probe` calls `at` twice on one line and `outer` calls `probe`
# twice on one line: the first of each is the one that panics, so the backtrace names it whatever
# line information the platform gives.

import support.geometry as geometry

# Each called from two places, so neither is inlined into its caller (a function called from one
# place is, and is then its caller's frame).
fn outer(g: geometry.Grid, i: int) -> int:
    return geometry.probe(g, i) + geometry.probe(g, 0)

fn warm(g: geometry.Grid) -> int:
    return outer(g, 0)

fn main():
    g = geometry.Grid([1, 2])
    println(warm(g) + warm(g))
    println(outer(g, 5))
