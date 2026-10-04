#$ test: run-pass
#$ rules: EXP-4, CTL-8, DRP-3, CTL-4
#$ profiles: debug, release, shipping
#$ stdout: drop 2
#$ stdout: drop 1
#$ stdout: 3
#$ stdout: drop 12
#$ stdout: drop 11
#$ stdout: drop 10
#$ stdout: 13
#$ stdout: drop 30
#$ stdout: 31
#$ stdout: drop 41
#$ stdout: drop 40
#$ stdout: 0
#$ stdout: drop 50
#$ stdout: drop 51
#$ stdout: drop 52
#$ stdout: 2
#$ stdout: drop 61
#$ stdout: defer
#$ stdout: drop 60
#$ stdout: 61
# D-504: a `return`, `break` or `continue` ends the temporaries of every
# statement it leaves, with the locals of every scope it leaves, the last made
# first; the return statement's own temporaries end before its `defer`s run.

struct Marker:
    pub n: int

    fn drop(mut self):
        println("drop", self.n)

fn number(marker: Marker) -> int:
    return marker.n

fn returned() -> int:
    return number(Marker(1)) + number(Marker(2))

fn from_a_for_bound() -> int:
    _kept = Marker(10)
    for i in 0..number(Marker(11)):
        _inner = Marker(12)
        return i + 13
    return 0

fn from_a_match_scrutinee(v: int) -> int:
    match number(Marker(30)) + v:
        30:
            return 31
        _:
            pass
    return 0

fn broken() -> int:
    total = 0
    outer: while true:
        for i in 0..number(Marker(40)):
            _kept = Marker(41)
            break outer
    return total

fn continued() -> int:
    count = 0
    for i in 0..3:
        match number(Marker(50 + i)):
            51:
                continue
            _:
                count += 1
    return count

fn deferred() -> int:
    _kept = Marker(60)
    defer:
        println("defer")
    return number(Marker(61))

fn main():
    println(returned())
    println(from_a_for_bound())
    println(from_a_match_scrutinee(0))
    println(broken())
    println(continued())
    println(deferred())
