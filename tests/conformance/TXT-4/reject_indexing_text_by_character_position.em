#$ test: compile-fail
#$ rules: TXT-4
## `[TXT-4]` - there is no indexing by character position, because it would be O(n): `s[n]` is not
## text, and slicing is by byte range.

fn main():
    s = "héllo"
    n = 2
    x = s[n]    #$ error[E2020]: cannot index `StrView`
    t = s[0]    #$ error[E2020]: cannot index `StrView`
    println(x, t)
