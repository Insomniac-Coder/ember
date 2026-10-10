#$ test: run-pass
#$ rules: DIA-14, TYP-17, LEX-19
#$ stdout: true <   7>
# Valid operands still undergo ordinary bound and format-spec checking.

fn same[T: Eq](a: T, b: T) -> bool:
    return a == b

fn show[T: Display](x: T) -> String:
    return f"<{x:>4}>"

fn main():
    println(same(3, 3), show(7))
