#$ test: run-pass
#$ rules: LEX-18
#$ stdout: 1 1.5 2 6.0
## `[LEX-18]` - `1.` followed by an identifier character is a method call on `1`; `1.0` is a float;
## after `.` the lexer reads an integer, never a float: `t.0.1` is two field accesses.

fn main():
    t = ((1, 2), 3)
    a = 1.to_string()
    f = 1.5
    s = 1.0 + 5.0
    println(a, f, t.0.1, s)
