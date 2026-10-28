#$ test: run-pass
#$ rules: BRW-1, BRW-11, BRW-21
#$ stdout: 1
#$ stdout: renamed
## design rules: R4, R8
## R4 (overlap, exception 1), R8 (f): `rename(ref mut LOG)` gives `s` the whole of `LOG`, which `rename`
## also names by its own name; the compiler proves they are one whole variable and compiles this call
## against a copy of `rename` in which `s` is `LOG` (verdict 88). It frees nothing: `LOG.clear()` empties
## the list and `s = ...` fills it again, so `LOG` holds one string. A part at a position not known when
## compiling stays refused (`reject_an_element_given_as_ref_mut_beside_its_whole_list`; the call check,
## E3031, for `rename(ref mut LOG[0])` in `reject_a_ref_mut_argument_into_what_the_callee_clears`).

static mut LOG: Array[String] = []

fn rename(ref mut s: Array[String]):
    LOG.clear()
    s = [String("renamed")]

fn main():
    LOG.push(String("a"))
    LOG.push(String("b"))
    rename(ref mut LOG)
    println(LOG.len())
    println(LOG[0])
