#$ test: compile-fail
#$ rules: BRW-21
#$ help: write `mine = copy of LOG[0]` and `rename(ref mut mine)` in place of line 16
## design rules: R4, R8
## R4, verdicts 53 and 87: the call check covers a `ref mut` parameter too: `rename` writes `s` after
## `LOG.clear()`, which frees the string `s` names when it is `LOG[0]`, so the call is refused (E3031,
## `V5/rules.md` section 3's message in this program's lines).

static mut LOG: Array[String] = []

fn rename(ref mut s: String):
    LOG.clear()
    s = String("renamed")

fn main():
    LOG.push(String("hello"))
    rename(ref mut LOG[0])    #$ error[E3031]: `rename` clears `LOG` (line 11) and then writes `s` (line 12), and `LOG[0]` is part of `LOG`, so `s` would point at freed memory; rename a copy of your own: write `mine = copy of LOG[0]` and `rename(ref mut mine)` in place of this line, or in `rename` write `s` before `LOG.clear()`
    println(LOG.len())
