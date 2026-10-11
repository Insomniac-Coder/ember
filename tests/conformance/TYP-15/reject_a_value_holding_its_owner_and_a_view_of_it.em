#$ test: compile-fail
#$ rules: TYP-15, DIA-7a

# `[TYP-15]`, shape B5 — a value whose one field would borrow another is
# `E3024`: moving the owner into it would move what the view points into. It
# was reported as an ordinary move while borrowed (`E3021`, shape B3), with
# advice to end the borrow, which cannot fix it.

@view
struct Parsed:
    text: String
    head: str

fn main():
    t = "hello world".to_string()
    p = Parsed(t, t[0..5])    #$ error[E3024]: `t` would be moved into this value while another of its fields borrows it
    println(p.head)
