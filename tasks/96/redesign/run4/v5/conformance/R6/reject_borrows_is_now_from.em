#$ test: compile-fail
#$ rules: ATT-1, PHIL-12
#$ help: fn first_word(src: StrView, sep: StrView) -> StrView from src:
## design rules: R6
## R6, rules section 3's removed words: `@borrows(src)` is gone; a result that points into one
## parameter says so after its type, `from src` (E0104; completeness G8).

@borrows(src)    #$ error[E0104]: `@borrows` is gone: remove this line and write `from src` after the result type: `fn first_word(src: StrView, sep: StrView) -> StrView from src:`
fn first_word(src: StrView, sep: StrView) -> StrView:
    end = src.find(sep).unwrap_or(src.len())
    return src[..end]

fn main():
    text: String = "let x"
    println(first_word(text, " "))
