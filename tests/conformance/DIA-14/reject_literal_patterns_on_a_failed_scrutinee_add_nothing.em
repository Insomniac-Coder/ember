#$ test: compile-fail
#$ rules: DIA-14, CTL-5
## `[DIA-14]` — a `match` whose scrutinee did not check reports that once: its literal patterns
## (here strings) add no "cannot match a value of type `<error>`" per arm, as they did (D2 of the
## first-week corpus, `tests/firstweek/23_recipe_scaler.em`).

fn unit(word: str) -> int:
    return match missing.trim():    #$ error[E1010]: cannot find `missing`
        "g" => 1
        "ml" => 2
        7 => 3
        _ => 0

fn main():
    println(unit("g"))
