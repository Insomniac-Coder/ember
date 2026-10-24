# A12: the same with `owned` on the parameter (what [CLO-7] says registries take).
class Button:
    handlers: Array[fn()] = []

    fn on_click(self, owned f: fn()):
        self.handlers.push(f)

    fn click(self):
        for h in self.handlers.iter_mut():
            h()

fn game() -> int:
    score = 0
    b = Button()
    b.on_click(fn(): score += 1)       # [CLO-15]: captures by move, so the handler bumps a copy
    b.click()
    return score

fn main():
    println(game())
