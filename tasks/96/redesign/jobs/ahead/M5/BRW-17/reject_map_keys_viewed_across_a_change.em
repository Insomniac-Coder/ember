# Clause BRW-17.7: "For keys viewed across a change of their map: E3060 ("`m.insert(owns
# String("z"), owns 1)` (line 8) changes `m`, whose keys `ks` (line 7) still views on line 9; use `ks`
# before line 8, or keep your own strings: write `ks = sorted(m.keys().copy_each())` in place of line
# 7")."
#$ test: compile-fail
#$ rules: BRW-17, CTL-1, LT-1a, LT-45
# The spec's message, with this file's line numbers.

fn main():
    m: Map[String, int] = {String("b"): 2, String("a"): 1}
    ks = sorted(m.keys())
    m.insert(owns String("z"), owns 1)
    println(ks)    #$ error[E3060]: `m.insert(owns String("z"), owns 1)` (line 12) changes `m`, whose keys `ks` (line 11) still views on line 13; use `ks` before line 12, or keep your own strings: write `ks = sorted(m.keys().copy_each())` in place of line 11
