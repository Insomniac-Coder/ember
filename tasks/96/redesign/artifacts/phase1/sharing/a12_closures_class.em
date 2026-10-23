# A12: the score in a class object, captured by a plain lambda / by `owned fn`.

class Button:
    handlers: Array[fn()] = []

    fn on_click(self, owned f: fn()):
        self.handlers.push(f)

    fn click(self):
        for h in self.handlers.iter_mut():
            h()

class Score:
    value: int = 0

fn game_class_owned() -> int:
    score = Score()
    b = Button()
    b.on_click(owned fn(): score.value += 1)
    b.click()
    return score.value

fn main():
    println(game_class_owned())
