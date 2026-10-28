#$ test: compile-fail
#$ rules: R12, R5, R10
#$ help: reset = fn() with owns players changes Player: players.clear()
## Verdict 55, R12 (rules.md lines 611-617): a function's own state needs no word, except ending a class
## object, which a `weakref` elsewhere may point at; a function that may end one names its type, so every
## call of it is seen to end players (R10). rules.md section 3's line 989 fourth message, naming `main`'s
## line too: after the fix `main` runs a `fn() changes Player` (R5, lines 349-351; R18, verdict 73).

class Player:
    hp: int = 10

fn main():
    players: Array[Player] = [Player(), Player()]
    w = weakref players[0]
    reset = fn() with owns players: players.clear()    #$ error[E2105]: this function ends the players it owns (`players.clear()`); say so on its first line: `reset = fn() with owns players changes Player: players.clear()`, and, since `main` runs it, add it to `main`'s first line: `fn main() changes Player:`
    reset()
    if w exists as p:
        println(p.hp)
    else:
        println("gone")
