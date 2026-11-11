#$ test: compile-fail
#$ rules: CTL-5, GRM-13
# `[GRM-13]` — a list pattern against a `String` or `StrView` is `E2020`, whose help gives `split`,
# and one against a `[T; N]` whose `N` can never equal the pattern's count is `E2020`, naming both.

fn quit(line: StrView) -> int:
    match line:
        ["quit"]:                   #$ error[E2020]: a list pattern cannot match `StrView`, which is text and not a list
            return 1
        _:
            return 0

fn pair(xs: [u8; 3]) -> int:
    match xs:
        [first, second]:            #$ error[E2020]: `[u8; 3]` has 3 elements, but this list pattern has 2
            return 1
        _:
            return 0

fn main():
    println(quit("quit"))
