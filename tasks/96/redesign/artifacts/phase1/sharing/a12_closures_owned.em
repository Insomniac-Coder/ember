# A12: `owned fn` written out: compiles; the handler bumps its own copy of `score`.

class Button:
    handlers: Array[fn()] = []

    fn on_click(self, owned f: fn()):
        self.handlers.push(f)

    fn click(self):
        for h in self.handlers.iter_mut():
            h()

fn game_owned() -> int:
    score = 0
    b = Button()
    b.on_click(owned fn(): score += 1)
    b.click()
    return score

fn main():
    println(game_owned())
