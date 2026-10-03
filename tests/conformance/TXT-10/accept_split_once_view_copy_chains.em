#$ test: run-pass
#$ rules: TXT-10, LT-20, LT-22, CG-C-3
#$ profiles: debug, release, shipping
#$ stdout: Some('left') Some('') None
#$ stdout: Some('a') Some('') Some('b') Some('') None
# A split's nested descriptors flow through ordinary local copies, a niche
# Option, a joined None path, and Some extraction before being returned.
# The C shape assertions accompany this fixture in milestones.rs.

@noinline
fn first_view(text: str, separator: str) -> Option[str]:
    found: Option[str] = None
    match text.split_once(separator):
        Some((before, after)):
            if after.is_empty():
                found = Some(after)
            else:
                copied = before
                found = Some(copied)
        None:
            pass
    match found:
        Some(value):
            returned = value
            return Some(returned)
        None:
            return None

fn main():
    println(first_view("left:right", ":"), first_view("left:", ":"), first_view("left", "!"))
    parts = "a,,b,".split(",")
    println(parts.next(), parts.next(), parts.next(), parts.next(), parts.next())
