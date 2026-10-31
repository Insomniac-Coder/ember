#$ test: compile-fail
#$ rules: MOD-3, TXT-1, STD-19, OWN-9
## `[MOD-3]` — a name Ember no longer has is `E1010`, with the one to write: the mode `owned` (`owns`),
## `str` (`StrView`) and `cstr` (`CStrView`), and `.copied()` / `.cloned()` (`.copy_each()`).

fn take(owned s: String) -> int:    #$ error[E1010]: `owned` is gone from Ember: the word that hands a value over is `owns`
    return s.len()

fn size(s: str) -> int:    #$ error[E1010]: `str` is gone from Ember: the view of text is `StrView`
    return s.len()

fn main():
    xs = [1, 2]
    total = xs.iter().copied().sum()    #$ error[E1010]: `.copied()` is not Ember: what `copy of` makes of each item is `.copy_each()`
    println(take(String("a")), size("b"), total)
