#$ test: run-pass
#$ rules: BRW-3, BCK-5, EXP-1
#$ profiles: debug, release, shipping
#$ stdout: 7 1 2
#$ stdout: aλé

fn append(mut values: Array[int], value: int):
    values.push(value)

fn main():
    values: Array[int] = [7]
    append(values, values.len() if values.len() > 0 else 0)
    values.push(values.len() if values.len() == 2 else 0)
    println(values[0], values[1], values[2])
    text: String = "a"
    text.insert(text.len() if text.len() > 0 else 0, 'é')
    text.insert(1, 'λ' if text.len() > 0 else 'x')
    println(text)
