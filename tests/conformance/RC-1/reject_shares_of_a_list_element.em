#$ test: compile-fail
#$ rules: RC-1, OWN-13
# `[RC-1]` — a list element has one owner, the list, and no second owner is made of one element
# (`E2122`): the message gives the list's own line with the word before each element.

class Player:
    hp: int = 100

fn main():
    players = [Player(), Player()]
    p = shares mut players[1]         #$ error[E2122]: `players[1]` has one owner, the list `players`, and no second owner can be made of one element; to let other names own the players, make the list share them from the start: write `players = [shares mut Player(), shares mut Player()]` in place of line 10
    println(p.hp)
