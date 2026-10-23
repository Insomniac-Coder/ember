# A11 with a class: handles are Copy, so the same program has no moves to get wrong.
class Msg:
    to: String
    body: String

fn send(owned m: Msg):
    pass

fn log(m: Msg):
    pass

fn notify(owned m: Msg):
    send(m)
    log(m)

fn main():
    notify(Msg(to="a", body="b"))
