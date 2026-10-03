#$ test: run-pass
#$ rules: TXT-10, EXP-1
#$ profiles: debug, release, shipping
#$ stdout: separator
#$ stdout: Some(('old', 'text')) changed
# The separator may repoint the source descriptor. The receiver's evaluated
# view is preserved before that expression runs, and each runs once.

fn separator(mut text: str) -> str:
    println("separator")
    text = "changed"
    return ":"

fn main():
    text: str = "old:text"
    result = text.split_once(separator(text))
    println(result, text)
