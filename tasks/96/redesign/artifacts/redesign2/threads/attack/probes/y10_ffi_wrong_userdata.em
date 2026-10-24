# PROPOSED syntax. Two callbacks of different closure types are registered with one C library, which (a bug
# in C, or a misidentified user_data in an inferred contract) passes the first one's user data to the second
# one's function pointer. Range and generation both pass: the slot is live. Without a type check, the
# second trampoline runs its body on the first closure's state.
fn main():
    a = c.on_event(fn(): game.score += 1)              # captures a Game handle
    b = c.on_text(fn(s: str): log.push(s.to_string()))  # captures an Array[String]
    c.mix_up()                                         # C calls on_text's pointer with on_event's user data
