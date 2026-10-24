# PROPOSED syntax. Tasks make result objects that keep a handle to a parent object: refused as written
# (two tasks write "the count of some Player"). Rust accepts r04_results_borrow_inputs.rs with a lifetime.
import std.thread

class Match:
    player: Player
    score: int

fn score_all(players: Array[Player]) -> Array[Match]:
    results: Array[Match] = []
    with scope = thread.scope():
        for p in players:
            scope.spawn(fn(): results.push(Match(player=p, score=p.name.len())))
    return results
