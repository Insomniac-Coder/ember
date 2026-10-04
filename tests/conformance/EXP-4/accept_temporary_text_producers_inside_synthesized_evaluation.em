#$ test: run-pass
#$ rules: EXP-1, EXP-4, DRP-3, TXT-10, SPN-1
#$ profiles: debug, release, shipping
#$ stdout: separator
#$ stdout: Some(('old', 'text')) changed
#$ stdout: Some(('é', 'left')) ('é', ':', 'left') Some('é')
# D-488 (the other agent's case). A temporary separator is consumed during the
# enclosing call, while an evaluated receiver keeps its previous descriptor.
# Temporary receivers are also safe when all returned views are consumed in
# that same source statement. Partition and get exercise other wrappers.

fn separator(mut text: str) -> String:
    println("separator")
    text = "changed"
    return ":"

fn make() -> String:
    return "é:left"

fn main():
    text: str = "old:text"
    result = text.split_once(separator(text))
    println(result, text)
    println(make().split_once(":"), make().partition(":"), make().get(0..2))
