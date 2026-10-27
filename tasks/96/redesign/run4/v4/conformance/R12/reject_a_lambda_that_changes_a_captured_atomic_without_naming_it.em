#$ test: compile-fail
#$ rules: R12, R5
#$ help: count_hit = fn() changes hits: hits.fetch_add(1)
## Verdict 75, R12 (rules.md lines 608-611): a real variable a function captures is named by its name,
## whatever its type, an `Atomic` included; naming by type (`changes Atomic[int]`) is for an `Atomic`
## reached through a link. A lambda that changes `hits` without saying so is refused (E2105), and the
## message names the variable. rules.md section 3's line 989 second message, in this program's names.

from std.sync import Atomic

fn main():
    hits = Atomic(0)
    count_hit = fn(): hits.fetch_add(1)    #$ error[E2105]: this function changes `hits`, a variable it uses from `main`; say so on its first line: `count_hit = fn() changes hits: hits.fetch_add(1)`
    count_hit()
    count_hit()
    println(hits.load())
