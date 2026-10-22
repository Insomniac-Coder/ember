# Bank accounts with deposits, withdrawals and transfers that fail with typed errors instead of exceptions.

enum BankError:
    InsufficientFunds(balance: int, requested: int)
    InvalidAmount(amount: int)
    AccountFrozen(owner: String)
    UnknownAccount(number: int)

class Account:
    let number: int
    owner: String
    pub(read) balance: int = 0
    frozen: bool = false
    history: Array[String] = []

    fn init(self, number: int, owned owner: String):
        self.number = number
        self.owner = owner

    fn deposit(self, amount: int) -> Result[int, BankError]:
        if amount <= 0:
            return Err(BankError.InvalidAmount(amount))
        if self.frozen:
            return Err(BankError.AccountFrozen(self.owner.clone()))
        self.balance += amount
        self.history.push(f"deposit {amount}")
        return Ok(self.balance)

    fn withdraw(self, amount: int) -> Result[int, BankError]:
        if amount <= 0:
            return Err(BankError.InvalidAmount(amount))
        if self.frozen:
            return Err(BankError.AccountFrozen(self.owner.clone()))
        if amount > self.balance:
            return Err(BankError.InsufficientFunds(self.balance, amount))
        self.balance -= amount
        self.history.push(f"withdraw {amount}")
        return Ok(self.balance)

class Bank:
    accounts: Map[int, Account] = {}
    next_number: int = 1000

    fn open(self, owner: str, initial: int) -> Result[Account, BankError]:
        account = Account(self.next_number, owner.to_string())
        self.next_number += 1
        account.deposit(initial)?
        self.accounts[account.number] = account
        return Ok(account)

    fn find(self, number: int) -> Result[Account, BankError]:
        if number in self.accounts:
            return Ok(self.accounts[number])
        return Err(BankError.UnknownAccount(number))

    fn transfer(self, src: int, dst: int, amount: int) -> Result[(), BankError]:
        from_acc = self.find(src)?
        to_acc = self.find(dst)?
        from_acc.withdraw(amount)?
        to_acc.deposit(amount)?
        return Ok(())

fn explain(e: BankError) -> String:
    return match e:
        InsufficientFunds(balance, requested) => f"insufficient funds: balance {balance}, requested {requested}"
        InvalidAmount(amount) => f"invalid amount {amount}"
        AccountFrozen(owner) => f"{owner}'s account is frozen"
        UnknownAccount(number) => f"no account {number}"

fn report(result: Result[(), BankError], what: str):
    match result:
        Ok(_) => println(f"ok: {what}")
        Err(e) => println(f"failed: {what}: {explain(e)}")

fn main():
    bank = Bank()
    alice = bank.open("alice", 500).unwrap()
    bob = bank.open("bob", 50).unwrap()
    if Err(e) = bank.open("carol", -5):
        println(f"could not open an account for carol: {explain(e)}")

    report(bank.transfer(alice.number, bob.number, 200), "alice -> bob 200")
    report(bank.transfer(bob.number, alice.number, 1000), "bob -> alice 1000")
    report(bank.transfer(alice.number, 4242, 10), "alice -> 4242 10")
    bob.frozen = true
    report(bank.transfer(alice.number, bob.number, 10), "alice -> bob 10")
    bob.frozen = false
    report(bank.transfer(bob.number, alice.number, 0), "bob -> alice 0")

    for account in [alice, bob]:
        println(f"{account.owner} #{account.number}: {account.balance}")
        for entry in account.history:
            println(f"    {entry}")
