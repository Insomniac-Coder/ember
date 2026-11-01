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

    fn init():
        this.add_room("hall", "A dusty hall. Doors lead north and east.")
        this.add_room("library", "Shelves full of mouldy books. A heavy door leads east.")
        this.add_room("kitchen", "Something smells awful in here.")
        this.add_room("vault", "Gold everywhere!")
        this.connect("hall", Direction.North, "library")
        this.connect("library", Direction.South, "hall")
        this.connect("hall", Direction.East, "kitchen")
        this.connect("kitchen", Direction.West, "hall")
        this.connect("library", Direction.East, "vault")
        this.connect("vault", Direction.West, "library")
        this.rooms["kitchen"].items.push(owns "key")
        this.rooms["library"].items.push(owns "candle")

    fn add_room(name: StrView, description: StrView):
        this.rooms[name] = Room(name=name.to_string(), description=description.to_string())

    fn connect(start: StrView, dir: Direction, end: StrView):
        this.rooms[start].exits[dir] = end.to_string()

    fn look():
        room = ref this.rooms[this.current]
        println(f"== {room.name} ==")
        println(room.description)
        if room.items.len() > 0:
            println(f"You see: {room.items.join(", ")}")

    fn go(dir: Direction):
        room = ref this.rooms[this.current]
        if dir in room.exits:
            next = ref room.exits[dir]
            if next == "vault" and "key" not in this.inventory:
                println("The vault door is locked. You need a key.")
                return
            this.current = next.clone()
            this.look()
            if next == "vault":
                println("You open the vault. You win!")
                this.finished = true
        else:
            println("You can't go that way.")

    fn take(item: StrView):
        room = ref mut this.rooms[this.current]
        if item.to_string() in room.items:
            room.items.retain(owns fn(i) => i != item)
            this.inventory.push(owns item.to_string())
            println(f"Taken: {item}")
        else:
            println(f"There is no {item} here.")

fn parse_direction(word: StrView) -> Option[Direction]:
    return match word:
        "north" | "n" => Some(Direction.North)
        "south" | "s" => Some(Direction.South)
        "east" | "e" => Some(Direction.East)
        "west" | "w" => Some(Direction.West)
        _ => None

fn run_command(game: Game, line: StrView):
    words = line.trim().to_lower().split(" ").map(owns fn(w) => w.to_string()).collect[Array[String]]()
    if words.len() == 0:
        return
    verb = ref words[0]
    if verb == "look":
        game.look()
    elif verb == "go" and words.len() > 1:
        if parse_direction(words[1]) exists as dir:
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
