# A12: a button handler that bumps the score.

class Button:
    handlers: Array[fn()] = []

    fn on_click(self, owned f: fn()):
        self.handlers.push(f)

    fn click(self):
        for h in self.handlers.iter_mut():
            h()

# Python: score is a variable the handler changes. Ember: the lambda is given to a stored
# (`owned`) parameter, so it captures `score` by copy ([CLO-15]); the handler bumps its own copy.
fn game() -> int:
    score = 0
    b = Button()
    b.on_click(fn(): score += 1)
    b.click()
    return score

# The same with `owned fn` written out.
fn game_owned() -> int:
    score = 0
    b = Button()
    b.on_click(owned fn(): score += 1)
    b.click()
    return score

# What works: the score lives in a class object; the handler retains it.
class Score:
    value: int = 0

fn game_class() -> int:
    score = Score()
    b = Button()
    b.on_click(fn(): score.value += 1)
    b.click()
    return score.value

fn main():
    println(game(), game_owned(), game_class())
