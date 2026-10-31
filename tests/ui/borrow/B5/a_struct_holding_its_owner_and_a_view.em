@view
struct Parsed:
    text: String
    head: StrView

fn main():
    t = "hello world".to_string()
    p = Parsed(t, t[0..5])
    println(p.head)
