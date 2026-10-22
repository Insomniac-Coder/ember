# Hangman with scripted guesses: reveal letters, count misses, draw the gallows and decide a win or a loss.

const MAX_MISSES: int = 6

class Hangman:
    secret: String
    guessed: Set[String] = Set()
    misses: int = 0

    fn init(self, secret: str):
        self.secret = secret.lower()

    fn masked(self) -> String:
        return " ".join([c if c in self.guessed else "_" for c in self.secret])

    fn guess(self, letter: str) -> String:
        letter = letter.lower()
        if letter.len() != 1:
            return "one letter at a time"
        if letter in self.guessed:
            return f"you already tried '{letter}'"
        self.guessed.add(letter)
        if letter in self.secret:
            return f"yes, {self.secret.count(letter)} of them"
        self.misses += 1
        return f"no '{letter}'"

    fn won(self) -> bool:
        return all(c in self.guessed for c in self.secret)

    fn lost(self) -> bool:
        return self.misses >= MAX_MISSES

    fn drawing(self) -> String:
        body = ["O", "/", "|", "\\", "/", "\\"]
        s = [body[i] if i < self.misses else " " for i in 0..6]
        return f"  +---+\n  |   {s[0]}\n  |  {s[1]}{s[2]}{s[3]}\n  |  {s[4]} {s[5]}\n -+-"

fn play(secret: str, guesses: str):
    game = Hangman(secret)
    println(f"new game: {game.masked()}")
    for letter in guesses:
        result = game.guess(letter)
        println(f"guess {letter}: {result}  ->  {game.masked()}  (misses {game.misses})")
        if game.won():
            println(f"solved! the word was '{game.secret}'")
            return
        if game.lost():
            println(game.drawing())
            println(f"hanged. the word was '{game.secret}'")
            return
    println(f"out of guesses with {game.masked()}")

fn main():
    play("ember", "eatrmbx")
    println("")
    play("compiler", "zqxeaouwvk")
    println("")
    play("Python", "pyTHon")
