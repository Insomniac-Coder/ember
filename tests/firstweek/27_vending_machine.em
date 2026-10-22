# A vending machine: insert coins, choose a slot, get change back, with an error for every way a sale can fail.

enum Coin:
    Ten
    Twenty
    Fifty
    Pound

fn coin_value(c: Coin) -> int:
    return match c:
        Ten => 10
        Twenty => 20
        Fifty => 50
        Pound => 100

enum VendError:
    UnknownSlot(slot: String)
    SoldOut(product: String)
    NotEnoughMoney(price: int, inserted: int)
    CannotMakeChange(amount: int)

struct Slot:
    product: String
    price: int
    stock: int

class Machine:
    slots: Map[String, Slot] = {}
    inserted: int = 0
    coins: Map[int, int] = {10: 5, 20: 5, 50: 2, 100: 0}
    sales: int = 0

    fn insert(self, c: Coin):
        value = coin_value(c)
        self.inserted += value
        self.coins[value] += 1

    fn make_change(self, amount: int) -> Result[Array[int], VendError]:
        change: Array[int] = []
        left = amount
        for value in [100, 50, 20, 10]:
            while left >= value and self.coins[value] > 0:
                self.coins[value] -= 1
                left -= value
                change.push(value)
        if left != 0:
            for c in change:
                self.coins[c] += 1
            return Err(VendError.CannotMakeChange(amount))
        return Ok(change)

    fn choose(self, code: str) -> Result[(String, Array[int]), VendError]:
        if code not in self.slots:
            return Err(VendError.UnknownSlot(code))
        slot = self.slots[code]
        if slot.stock == 0:
            return Err(VendError.SoldOut(slot.product))
        if self.inserted < slot.price:
            return Err(VendError.NotEnoughMoney(slot.price, self.inserted))
        change = self.make_change(self.inserted - slot.price)?
        self.slots[code].stock -= 1
        self.inserted = 0
        self.sales += slot.price
        return Ok((slot.product, change))

    fn refund(self) -> int:
        amount = self.inserted
        self.inserted = 0
        return amount

fn main():
    m = Machine()
    m.slots["A1"] = Slot(product="crisps", price=60, stock=2)
    m.slots["A2"] = Slot(product="chocolate", price=85, stock=1)
    m.slots["B1"] = Slot(product="water", price=100, stock=0)

    orders: Array[(Array[Coin], str)] = [
        ([Coin.Fifty, Coin.Twenty], "A1"),
        ([Coin.Pound], "A2"),
        ([Coin.Pound], "B1"),
        ([Coin.Twenty, Coin.Twenty], "A1"),
        ([Coin.Pound], "C9"),
        ([Coin.Pound], "A2"),
    ]
    for coins, code in orders:
        for c in coins:
            m.insert(c)
        println(f"inserted {m.inserted}p, choosing {code}")
        match m.choose(code):
            Ok((product, change)):
                println(f"  -> {product}, change {change}")
            Err(e):
                match e:
                    UnknownSlot(s) => println(f"  there is no slot {s}")
                    SoldOut(p) => println(f"  {p} is sold out")
                    NotEnoughMoney(price, have) => println(f"  costs {price}p, you put in {have}p")
                    CannotMakeChange(amount) => println(f"  cannot give {amount}p change")
                println(f"  refunded {m.refund()}p")
    println(f"total sales: {m.sales}p")
