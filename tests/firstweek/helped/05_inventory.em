# A shop inventory: add stock, sell items with errors for unknown items or short stock, and report the stock value.

struct Item:
    name: String
    price: float
    quantity: int

enum StockError:
    UnknownItem(name: String)
    NotEnough(name: String, wanted: int, have: int)

class Inventory:
    items: Map[String, Item] = {}

    fn add(name: StrView, price: float, quantity: int):
        if name in this.items:
            this.items[name].quantity += quantity
        else:
            this.items[name] = Item(name=name.to_string(), price=price, quantity=quantity)

    fn sell(name: StrView, quantity: int) -> Result[float, StockError]:
        if name not in this.items:
            return Err(StockError.UnknownItem(name.to_string()))
        item = ref this.items[name]
        if item.quantity < quantity:
            return Err(StockError.NotEnough(name.to_string(), quantity, item.quantity))
        price = item.price
        this.items[name].quantity -= quantity
        return Ok(price * quantity as float)

    fn total_value() -> float:
        return sum(item.price * item.quantity as float for item in this.items.values())

    fn low_stock(threshold: int) -> Array[String]:
        return [name.clone() for name, item in this.items.items() if item.quantity < threshold]

    fn report():
        println("stock:")
        for name in sorted(this.items.keys()):
            item = ref this.items[name]
            println(f"  {name:<8} {item.quantity:>4} x {item.price:.2f}")

fn describe(err: StockError) -> String:
    return match err:
        UnknownItem(name) => f"no such item: {name}"
        NotEnough(name, wanted, have) => f"only {have} {name} left, wanted {wanted}"

fn main():
    shop = Inventory()
    shop.add("apple", 0.5, 100)
    shop.add("pear", 0.75, 20)
    shop.add("melon", 3.0, 4)
    shop.add("apple", 0.5, 50)
    shop.report()

    orders = [("apple", 30), ("melon", 5), ("kiwi", 1), ("pear", 18), ("melon", 4)]
    takings = 0.0
    for name, qty in orders:
        match shop.sell(name, qty):
            Ok(amount):
                takings += amount
                println(f"sold {qty} {name} for {amount:.2f}")
            Err(e):
                println(f"cannot sell: {describe(e)}")

    println(f"takings: {takings:.2f}")
    println(f"stock value now: {shop.total_value():.2f}")
    low = shop.low_stock(5)
    if low.len() > 0:
        println(f"reorder: {low}")
    shop.report()
