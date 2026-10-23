# A12: the score in a class object, captured by a plain lambda.

class Button:
    handlers: Array[fn()] = []

    fn on_click(self, owned f: fn()):
        self.handlers.push(f)

class Score:
    value: int = 0

fn game_class() -> int:
    score = Score()
    b = Button()
    b.on_click(fn(): score.value += 1)
    return score.value

fn main():
    println(game_class())
