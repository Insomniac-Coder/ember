# Simulate a supermarket checkout queue minute by minute with a seeded random generator and report the waiting times.

struct Rng:
    state: int

    fn next() -> int:
        this.state = (this.state * 1103515245 + 12345) % 2147483648
        return this.state

    fn chance(percent: int) -> bool:
        return this.next() % 100 < percent

    fn between(low: int, high: int) -> int:
        return low + this.next() % (high - low + 1)

struct Customer:
    id: int
    arrived: int
    items: int

class Checkout:
    name: String
    serving: Option[Customer] = None
    remaining: int = 0
    served: int = 0

    fn init(owns name: String):
        this.name = name

    fn is_free() -> bool:
        return this.serving is None

    fn start(owns customer: Customer):
        this.remaining = customer.items
        this.serving = Some(customer)

    fn tick() -> bool:
        if this.serving is None:
            return false
        this.remaining -= 1
        if this.remaining <= 0:
            this.serving = None
            this.served += 1
            return true
        return false

fn main():
    rng = Rng(state=42)
    queue: Array[Customer] = []
    tills = [Checkout("till 1"), Checkout("till 2")]
    waits: Array[int] = []
    next_id = 1
    longest_queue = 0

    for minute in 0..120:
        if rng.chance(45):
            queue.push(Customer(id=next_id, arrived=minute, items=rng.between(1, 6)))
            next_id += 1
        for till in tills.iter_mut():
            till.tick()
            if till.is_free() and queue.len() > 0:
                customer = queue.remove(0)
                waits.push(minute - customer.arrived)
                till.start(customer)
        longest_queue = max(longest_queue, queue.len())
        if minute % 30 == 0:
            println(f"minute {minute:>3}: {queue.len()} waiting")

    println(f"customers arrived: {next_id - 1}")
    for till in tills:
        println(f"{till.name} served {till.served}")
    if waits.len() > 0:
        average = sum(waits) as float / waits.len() as float
        println(f"average wait: {average:.2f} minutes, longest wait: {waits.iter().max().unwrap()} minutes")
    println(f"longest queue: {longest_queue}, still waiting at closing time: {queue.len()}")
