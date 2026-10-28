#$ test: compile-fail
#$ rules: R4, R8
#$ help: mine = copy of LOG[0]
## R4, verdict 53 as the rules apply it to every view parameter, a `ref mut` one included (V4
## changes.md section 2.1, for the owner): `rename` changes `s` after `LOG.clear()`, which frees the
## string `s` names when it is `LOG[0]`, so the call is refused (E3031).

static mut LOG: Array[String] = []

fn rename(ref mut s: String) changes LOG:
    LOG.clear()
    s = String("renamed")

fn main() changes LOG:
    LOG.push(String("hello"))
    rename(ref mut LOG[0])    #$ error[E3031]: `rename` changes `LOG` and changes `s` after `LOG.clear()` (line 11), and `LOG[0]` is part of `LOG`; give it a string of your own: write `mine = copy of LOG[0]` and `rename(ref mut mine)` in place of this line, or in `rename` change `s` before `LOG.clear()`
    println(LOG.len())
