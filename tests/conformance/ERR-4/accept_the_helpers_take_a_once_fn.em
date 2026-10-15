#$ test: run-pass
#$ rules: ERR-4, CLO-6
#$ stdout: Some('hi!')
#$ 7
#$ Ok(4)
# `[ERR-4]` (ODR-025) — the `Option`/`Result` methods that take a function call it at most once, so
# they take a `once fn`: a closure may give away what it captured (D6, `N3`, formerly refused).

fn join(owned a: String, tail: str) -> String:
    a.push_str(tail)
    return a

fn main():
    tail = "!"
    name: Option[String] = Some("hi".to_string())
    println(name.map(owned fn(s) => join(s, tail)))
    keep = "kept".to_string()
    failed: Result[int, String] = Err("bad".to_string())
    println(failed.unwrap_or_else(owned fn(e) => e.len() + keep.len()))
    word = "word".to_string()
    fine: Result[int, String] = Ok(1)
    println(fine.map(owned fn(v) => v + word.len() - 1))
