# The dice game Pig for three computer players with different strategies, driven by a seeded random generator.

const TARGET: int = 100

struct Dice:
    seed: int

    fn roll(mut self) -> int:
        self.seed = (self.seed * 1103515245 + 12345) % 2147483648
        return (self.seed // 65536) % 6 + 1

enum Strategy:
    HoldAt(limit: int)
    RollTimes(count: int)

class Player:
    name: String
    strategy: Strategy
    score: int = 0
    turns: int = 0

    fn init(self, owned name: String, strategy: Strategy):
        self.name = name
        self.strategy = strategy

    fn wants_to_roll(self, turn_total: int, rolls: int) -> bool:
        return match self.strategy:
            HoldAt(limit) => turn_total < limit
            RollTimes(count) => rolls < count

fn play_turn(player: Player, mut dice: Dice) -> int:
    turn_total = 0
    rolls = 0
    while player.wants_to_roll(turn_total, rolls):
        r = dice.roll()
        rolls += 1
        if r == 1:
            println(f"  {player.name} rolls a 1 and loses {turn_total}")
            return 0
        turn_total += r
        if player.score + turn_total >= TARGET:
            break
    println(f"  {player.name} holds with {turn_total} after {rolls} rolls")
    return turn_total

fn main():
    dice = Dice(seed=2024)
    players = [
        Player("cautious", Strategy.HoldAt(15)),
        Player("greedy", Strategy.HoldAt(30)),
        Player("counter", Strategy.RollTimes(3)),
    ]
    round = 1
    winner: Option[Player] = None
    while winner is None and round <= 50:
        println(f"round {round}")
        for p in players:
            p.score += play_turn(p, dice)
            p.turns += 1
            if p.score >= TARGET:
                winner = Some(p)
                break
        round += 1

    for p in players:
        println(f"{p.name}: {p.score} points in {p.turns} turns")
    if Some(w) = winner:
        println(f"{w.name} wins!")
    else:
        println("no winner after 50 rounds")

    # how fair is the die?
    counts = [0 for _ in 0..7]
    for _ in 0..6000:
        counts[dice.roll()] += 1
    for face in 1..=6:
        println(f"{face}: {counts[face]}")
