# Clause EFF-9.1: "`Arithmetic` covers (...) the compare of an owner count or a weak count with its
# limit before it grows; (...) `Aliasing` covers the compare of `assert_disjoint` and
# `check_disjoint` (`[DSJ-1]`) and the test of a `runtime` name's fact (...), the two run-time alias
# checks there are; `Stale` covers handle generations and the test of a target's header that opens a
# `weakref` link (`exists as`)."
#$ test: run-pass
#$ rules: EFF-9, EFF-12, EFF-16, RC-1, WK-11
#$ stdout: 4
#$ stdout: 9
# A `weakref` link opened (its header test is `Stale`), a number written (its overflow check is
# `Arithmetic`) and `shares` owners made (their count's compare is `Arithmetic`): none is an
# `Aliasing` check, so `@no_alias_checks` holds, and `@nopanic(explicit)` holds where nothing
# allocates (`[EFF-12]`, `[EFF-16]`: allocation failure is a panic it forbids).

class Player:
    hp: int = 5

class Camera:
    target: weakref Player = None

@no_alias_checks
@nopanic(explicit)
fn hurt(ref mut p: Player):
    p.hp -= 1

@no_alias_checks
@nopanic(explicit)
fn peek(cam: Camera) -> int:
    if cam.target exists as t:
        return t.hp
    return 0

@no_alias_checks
fn share_twice() -> int:
    a = Player()
    b = shares a
    c = shares b
    return c.hp + 4

fn main():
    p = Player()
    cam = Camera()
    cam.target = weakref p
    hurt(ref mut p)
    println(peek(cam))
    println(share_twice())
