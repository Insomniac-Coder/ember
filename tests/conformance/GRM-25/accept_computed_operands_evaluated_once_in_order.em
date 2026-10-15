#$ test: run-pass
#$ rules: GRM-25, EXP-1
#$ profiles: debug, release
#$ stdout: true abcd
#$ false ab
#$ true m
#$ true true
#$ true mnb
# `[GRM-25]` — in a chain, each operand is evaluated at most once and left to right, and evaluation
# stops at the first false comparison: the log records each call. A middle operand of literals takes
# its type from its neighbours (`u8` here), and a chain compares `String`s as well as numbers.

class Log:
    text: String = String()

fn val(log: Log, name: str, v: int) -> int:
    log.text += name
    return v

fn word(log: Log, w: str) -> String:
    log.text += w
    return w.to_string()

fn main():
    log = Log()
    println(val(log, "a", 1) < val(log, "b", 2) < val(log, "c", 3) < val(log, "d", 4), log.text)
    log.text = String()
    println(val(log, "a", 1) < val(log, "b", 0) < val(log, "c", 3), log.text)
    log.text = String()
    println(0 <= val(log, "m", 5) < 10, log.text)
    x: u8 = 3
    println(x < 2 * 3 < 10, 1 < val(log, "n", 2) <= 2)
    println("a" < word(log, "b") < "c", log.text)
