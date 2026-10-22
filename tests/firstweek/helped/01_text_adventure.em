# A tiny text adventure: walk between rooms, pick up items, and win by opening the vault with the key.

enum Direction:
    North
    South
    East
    West

struct Room:
    name: String
    description: String
    exits: Map[Direction, String] = {}
    items: Array[String] = []

class Game:
    rooms: Map[String, Room] = {}
    current: String = "hall"
    inventory: Array[String] = []
    finished: bool = false

    fn init(self):
        self.add_room("hall", "A dusty hall. Doors lead north and east.")
        self.add_room("library", "Shelves full of mouldy books. A heavy door leads east.")
        self.add_room("kitchen", "Something smells awful in here.")
        self.add_room("vault", "Gold everywhere!")
        self.connect("hall", Direction.North, "library")
        self.connect("library", Direction.South, "hall")
        self.connect("hall", Direction.East, "kitchen")
        self.connect("kitchen", Direction.West, "hall")
        self.connect("library", Direction.East, "vault")
        self.connect("vault", Direction.West, "library")
        self.rooms["kitchen"].items.push("key")
        self.rooms["library"].items.push("candle")

    fn add_room(self, name: str, description: str):
        self.rooms[name] = Room(name=name.to_string(), description=description.to_string())

    fn connect(self, start: str, dir: Direction, end: str):
        self.rooms[start].exits[dir] = end.to_string()

    fn look(self):
        room = ref self.rooms[self.current]
        println(f"== {room.name} ==")
        println(room.description)
        if room.items.len() > 0:
            println(f"You see: {room.items.join(", ")}")

    fn go(self, dir: Direction):
        room = ref self.rooms[self.current]
        if dir in room.exits:
            next = ref room.exits[dir]
            if next == "vault" and "key" not in self.inventory:
                println("The vault door is locked. You need a key.")
                return
            self.current = next.clone()
            self.look()
            if next == "vault":
                println("You open the vault. You win!")
                self.finished = true
        else:
            println("You can't go that way.")

    fn take(self, item: str):
        room = ref mut self.rooms[self.current]
        if room.items.iter().any(fn(e) => e == item):
            room.items.retain(fn(i) => i != item)
            self.inventory.push(item.to_string())
            println(f"Taken: {item}")
        else:
            println(f"There is no {item} here.")

fn parse_direction(word: str) -> Option[Direction]:
    if word == "north" or word == "n":
        return Some(Direction.North)
    if word == "south" or word == "s":
        return Some(Direction.South)
    if word == "east" or word == "e":
        return Some(Direction.East)
    if word == "west" or word == "w":
        return Some(Direction.West)
    return None

fn run_command(game: Game, line: str):
    words = line.trim().to_lower().split(" ").map(fn(w) => w.to_string()).collect[Array[String]]()
    if words.len() == 0:
        return
    verb = words[0].clone()
    if verb == "look":
        game.look()
    elif verb == "go" and words.len() > 1:
        if Some(dir) = parse_direction(words[1]):
            game.go(dir)
        else:
            println(f"I don't know the direction '{words[1]}'.")
    elif verb == "take" and words.len() > 1:
        game.take(words[1])
    elif verb == "inventory":
        println(f"You carry: {game.inventory}")
    else:
        println(f"I don't understand '{line}'.")

fn main():
    game = Game()
    game.look()
    commands = ["go north", "take candle", "go east", "go south", "go east",
                "take key", "inventory", "dance", "go west", "go north", "go east"]
    for command in commands:
        println(f"> {command}")
        run_command(game, command)
        if game.finished:
            break
