#$ test: compile-fail
#$ rules: SPN-1, LT-1
# The coercion creates a genuine borrow of `text`; it does not turn a value
# parameter into caller-owned view storage. Only a view-typed parameter can
# supply a returned view's region, so this borrow ends with the callee frame.

fn borrow_text(text: String) -> str:
    return text  #$ error[E3060]: `text` does not live long enough

fn main():
    s: String = String()
    s.push_str("hi")
    println(borrow_text(s))
