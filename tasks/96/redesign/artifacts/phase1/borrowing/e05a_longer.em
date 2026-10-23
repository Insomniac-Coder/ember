# A5 (i): returning one of two string views, no annotation.
fn longer(a: str, b: str) -> str:
    return a if a.len() >= b.len() else b

fn main():
    println(longer("sword", "axe"))
