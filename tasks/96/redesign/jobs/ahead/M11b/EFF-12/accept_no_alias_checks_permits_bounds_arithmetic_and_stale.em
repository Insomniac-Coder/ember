# Clause EFF-12.1: "`@no_alias_checks` permits `RuntimeCheck(Bounds)`, `(Arithmetic)` and
# `(Stale)`"; the message's fix: "blend from a copy, which needs no check"
#$ test: run-pass
#$ rules: EFF-12, EFF-9, DIA-25
#$ stdout: 3.0
#$ stdout: 2
# The message's line under `@no_alias_checks`, with an index, a division and a `weakref` opening.

class Player:
    hp: int = 4

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] += src[i]

@no_alias_checks
fn mix(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    old = copy of tracks[src_name]
    blend(ref mut tracks[dst_name].as_mut_span(), old.as_span())

@no_alias_checks
fn half_of_first(xs: Span[int], w: weakref Player) -> int:
    if w exists as p:
        return xs[0] // p.hp
    return 0

fn main():
    tracks: Map[String, Array[float]] = Map()
    tracks.insert(owns String("a"), owns [1.0])
    tracks.insert(owns String("b"), owns [2.0])
    mix(ref mut tracks, "a", "b")
    println(tracks["a"][0])
    p = Player()
    println(half_of_first([8], weakref p))
