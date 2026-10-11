struct Parsed:
    text: String
    head_end: int

fn main():
    t = "hello world".to_string()
    p = Parsed(t, 5)
    println(p.text[0..p.head_end])
