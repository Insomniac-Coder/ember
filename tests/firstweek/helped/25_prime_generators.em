# Generators: primes, Fibonacci numbers and FizzBuzz produced lazily and consumed by loops and helpers.

gen fn naturals(start: int) -> Generator[int]:
    n = start
    while true:
        yield n
        n += 1

gen fn primes() -> Generator[int]:
    found: Array[int] = []
    for n in naturals(2):
        is_prime = true
        for p in found:
            if p * p > n:
                break
            if n % p == 0:
                is_prime = false
                break
        if is_prime:
            found.push(n)
            yield n

gen fn fibonacci(limit: int) -> Generator[int]:
    a, b = 0, 1
    while a <= limit:
        yield a
        a, b = b, a + b

gen fn fizzbuzz(n: int) -> Generator[String]:
    for i in 1..=n:
        if i % 15 == 0:
            yield "FizzBuzz"
        elif i % 3 == 0:
            yield "Fizz"
        elif i % 5 == 0:
            yield "Buzz"
        else:
            yield f"{i}"

fn take[I: Iterator[Item = int]](owned g: I, n: int) -> Array[int]:
    result: Array[int] = []
    for x in g:
        if result.len() >= n:
            break
        result.push(x)
    return result

fn collatz_steps(start: int) -> int:
    steps = 0
    n = start
    while n != 1:
        n = n // 2 if n % 2 == 0 else 3 * n + 1
        steps += 1
    return steps

fn main():
    println(f"first 15 primes: {take(primes(), 15)}")
    fibs = [f for f in fibonacci(1000)]
    println(f"fibonacci up to 1000: {fibs}")
    even_sum = sum(f for f in fibonacci(4_000_000) if f % 2 == 0)
    println(f"sum of the even fibonacci numbers up to four million: {even_sum}")
    println(fizzbuzz(20).join(", "))

    for p in primes():
        if p > 100:
            println(f"first prime above 100: {p}")
            break

    for i, p in primes().enumerate():
        if i == 99:
            println(f"the 100th prime is {p}")
            break

    longest = 0
    best = 0
    for n in 1..30:
        s = collatz_steps(n)
        if s > longest:
            longest = s
            best = n
    println(f"below 30, {best} takes the most collatz steps: {longest}")
