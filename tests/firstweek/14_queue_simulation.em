# Simulate a supermarket checkout queue minute by minute with a seeded random generator and report the waiting times.

struct Rng:
    state: int

    fn next(mut self) -> int:
        self.state = (self.state * 1103515245 + 12345) % 2147483648
        return self.state

    fn chance(mut self, percent: int) -> bool:
        return self.next() % 100 < percent

    fn between(mut self, low: int, high: int) -> int:
        return low + self.next() % (high - low + 1)

struct Customer:
    id: int
    arrived: int
    items: int

class Checkout:
    name: String
    serving: Option[Customer] = None
    remaining: int = 0
    served: int = 0

    fn init(self, owned name: String):
        self.name = name

    fn is_free(self) -> bool:
        return self.serving is None

    fn start(self, owned customer: Customer):
        self.remaining = customer.items
        self.serving = Some(customer)

    fn tick(self) -> bool:
        if self.serving is None:
            return False
        self.remaining -= 1
        if self.remaining <= 0:
            self.serving = None
            self.served += 1
            return True
        return False

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
        for till in tills:
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
        average = float(sum(waits)) / float(waits.len())
        println(f"average wait: {average:.2f} minutes, longest wait: {max(waits)} minutes")
    println(f"longest queue: {longest_queue}, still waiting at closing time: {queue.len()}")
