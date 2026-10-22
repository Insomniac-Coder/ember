#$ test: run-pass
#$ rules: CG-C-10
#$ profiles: debug
#$ stdout: 9
#$ assert-c: contains("{"em_8geometry4Grid2at", "geometry.Grid.at", "geometry.em", 7u},")
#$ assert-c: contains("{"em_8geometry5probe", "geometry.probe", "geometry.em", 10u},")
#$ assert-c: contains("{"em_8geometry5probe", "em_8geometry4Grid2at", "geometry.em", 11u, 17u},")
#$ assert-c: contains("{"em_8geometry5probe", "em_8geometry4Grid2at", "geometry.em", 11u, 27u},")
#$ assert-c: contains("{"em_5outer", "em_8geometry5probe", "accept_frames_are_named_by_path_and_calls_by_place.em", 22u, 30u},")
#$ assert-c: contains("ember_backtrace_sites(em_call_sites, ")
# `[CG-C-10]`: what a stack trace is printed from. Outside `release` the program names each of its
# functions by its Ember path, a method by its type's path and its name (`geometry.Grid.at`), and
# lists each call one of them makes to another, with where the call is written (`file:line:col`,
# the column of its last argument, of the call itself when it has none): `probe` calls `Grid.at`
# twice on line 11, `g.at(i)` and `g.at(0)`, at columns 17 and 27. Each
# function is called from two places, so none is inlined into its caller. The run-fail case beside
# this one shows a backtrace printed from these tables.

import geometry

fn outer(g: geometry.Grid, i: int) -> int:
    return geometry.probe(g, i) + geometry.probe(g, 0)

fn main():
    g = geometry.Grid([1, 2])
    println(outer(g, 1) + outer(g, 0))
