#$ test: compile-fail
#$ rules: LEX-10, GRM-37
#$ error[E0104]: `#! module overflow(wrap)` is a directive, and directives come first; move it above `fn main()`
## `[LEX-10]` - the same below an item, even where the item's indented block has just ended.

fn main():
    println(1)

#! module overflow(wrap)
