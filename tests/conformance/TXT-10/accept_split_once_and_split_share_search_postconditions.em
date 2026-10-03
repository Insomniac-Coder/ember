#$ test: run-pass
#$ rules: TXT-10, TXT-4, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: (2, 4) (0, 0) (0, 3) (-1, -1) (1, 1) (0, 0)
#$ stdout: (4, 2) (1, 0) (1, 3)
#$ assert-c: contains("ember_str_search(")
#$ assert-c-count: contains("ember_str_is_char_boundary(") == 0
#$ assert-c-count: contains("ember_panic_bounds(") == 0
#$ assert-c-count: contains("ember_ck_add_i64(") == 2
# A successful search of valid text proves the two borrowed sides already
# lie on scalar boundaries. Dynamic separators include Unicode and NUL;
# empty split_once still succeeds, and Split retains every empty part.
# The two unconstrained totals remain checked.

@noinline
fn first_lengths(text: str, separator: str) -> (int, int):
    match text.split_once(separator):
        Some((before, after)):
            return (before.len(), after.len())
        None:
            return (-1, -1)

@noinline
fn totals(text: str, separator: str) -> (int, int):
    count = 0
    length = 0
    for part in text.split(separator):
        count += 1
        length += part.len()
    return (count, length)

fn main():
    empty: String = String.with_capacity(0)
    println(first_lengths("é🦀tail", "🦀"), first_lengths("é", "é"), first_lengths("abc", ""), first_lengths("é", "x"), first_lengths("a\0b", "\0"), first_lengths(empty.as_str(), ""))
    println(totals("🦀é🦀🦀", "🦀"), totals(empty.as_str(), ","), totals("abc", "xyz"))
