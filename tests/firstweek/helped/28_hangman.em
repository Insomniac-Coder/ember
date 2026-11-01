# Hangman with scripted guesses: reveal letters, count misses, draw the gallows and decide a win or a loss.

const MAX_MISSES: int = 6

class Hangman:
    secret: String
    guessed: Set[String] = Set()
    misses: int = 0

    fn init(secret: StrView):
        this.secret = secret.to_lower()

    fn masked() -> String:
        return [c if c.to_string() in this.guessed else '_' for c in this.secret].join(" ")

    fn guess(letter: StrView) -> String:
        new_letter = letter.to_lower()
        if new_letter.len() != 1:
            return "one letter at a time"
        if new_letter in this.guessed:
            return f"you already tried '{new_letter}'"
        this.guessed.add(owns new_letter.to_string())
        if new_letter in this.secret:
            return f"yes, {this.secret.count(new_letter)} of them"
        this.misses += 1
        return f"no '{new_letter}'"

    fn won() -> bool:
        return all(c.to_string() in this.guessed for c in this.secret)

    fn lost() -> bool:
        return this.misses >= MAX_MISSES

    fn drawing() -> String:
        body = ["O", "/", "|", "\\", "/", "\\"]
        s = [body[i] if i < this.misses else " " for i in 0..6]
        return f"  +---+\n  |   {s[0]}\n  |  {s[1]}{s[2]}{s[3]}\n  |  {s[4]} {s[5]}\n -+-"

fn play(secret: StrView, guesses: StrView):
    game = Hangman(secret)
    println(f"new game: {game.masked()}")
    for letter in guesses:
        result = game.guess(letter.to_string())
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
