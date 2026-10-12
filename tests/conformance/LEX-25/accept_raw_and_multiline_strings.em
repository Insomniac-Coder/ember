#$ test: run-pass
#$ rules: LEX-25
#$ stdout: C:\no\escape
#$ stdout: say "hi"
#$ stdout: true 10
## `[LEX-25]` — a raw string has no escapes, and `r#"…"#` may hold `"`; a
## multiline string drops an empty first line and the common leading spaces.

fn main():
    println(r"C:\no\escape")
    println(r#"say "hi""#)
    text = """
    one
      two
    """
    println(text == "one\n  two\n", text.len())
