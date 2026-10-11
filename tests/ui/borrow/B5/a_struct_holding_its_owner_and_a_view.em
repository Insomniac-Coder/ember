@view
struct Parsed:
    text: String
    head: str

fn main():
    t = "hello world".to_string()
    p = Parsed(t, t[0..5])
    println(p.head)
