# Clause BRW-1.9: "a change that may reshape it ends it instead (`E3060`, at its later use)."
#$ test: compile-fail
#$ rules: BRW-1, BRW-17, BCK-3
# Codes only: the spec gives no text here (shape B7).

fn a_text_view():
    s = String("ab ")
    w = s.trim()
    s.push_str("c")
    println(w)    #$ error[E3060]

fn a_span():
    xs = [1, 2, 3]
    v = xs.as_span()
    xs.clear()
    println(v.len())    #$ error[E3060]

fn main():
    a_text_view()
    a_span()
