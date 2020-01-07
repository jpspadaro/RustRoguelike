use tcod::colors::*;
use tcod::console::*;

//Actual size of the window

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 45;

const LIMIT_FPS: i32 = 20; //20 FPS max

const COLOR_DARK_WALL: Color = Color {r: 0, g: 0, b: 100};
const COLOR_DARK_GROUND: Color = Color {r: 0, g: 50, b: 150,};

///A tile of the map and its properties
#[derive(Clone, Copy, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
	    blocked: false,
	    block_sight: false,
	}
    }

    pub fn wall() -> Self {
        Tile {
            blocked: false,
	    block_sight: false,
        }
    }
}

#[derive(Debug)]
struct Tcod {
    root: Root,
    con: Offscreen,
}

type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
}

fn make_map() -> Map {
    // fill map with unblocked tiles
    let mut map = vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    //Two test pillars for map
    map[30][22] = Tile::wall();
    map[50][22] = Tile::wall();

    map
}

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object]){
    for object in objects {
        object.draw(&mut tcod.con);
    }
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
	    let wall = game.map[x as usize][y as usize].block_sight;
	    if wall {
	        tcod.con.set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
	    } else {
	        tcod.con.set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
            }
	}
    }
    blit( &tcod.con, (0,0), (MAP_WIDTH, MAP_HEIGHT), &mut tcod.root, (0,0), 1.0, 1.0,);
}

struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object { x, y, char, color }
    }

    ///move by a given amount
    pub fn move_by(&mut self, dx: i32, dy: i32) {
         self.x += dx;
	 self.y += dy;
    }

    ///set color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
	con.put_char(self.x, self.y, self.char, BackgroundFlag:: None);
    }
}

fn handle_keys(tcod: &mut Tcod, player: &mut Object) -> bool {
    use tcod::input::Key;  
    use tcod::input::KeyCode::*;

    //handle keys here

    let key = tcod.root.wait_for_keypress(true);
    match key {
	//movement keys
	Key { code: Up, .. } => player.move_by(0, -1),
	Key { code: Down, .. } => player.move_by(0, 1),
	Key { code: Left, .. } => player.move_by(-1, 0),
	Key { code: Right, .. } => player.move_by(1, 0),

	Key {
	    code: Enter,
	    alt: true,
	    ..
	} => {
	    //Full Screen Mode (Alt + Enter)
	    let fullscreen = tcod.root.is_fullscreen();
	    tcod.root.set_fullscreen(!fullscreen);
	}
	Key {code: Escape, .. } => return true,

	_ => {}
    }
    
    
    false
}


fn main() {
    tcod::system::set_fps(LIMIT_FPS);

    let root = Root::initializer()
	.font("arial10x10.png", FontLayout::Tcod)
	.font_type(FontType::Greyscale)
	.size(SCREEN_WIDTH, SCREEN_HEIGHT)
	.title("RUST/libtcod tutorial")
	.init();

    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);

    let mut tcod = Tcod { root, con };

    let mut player_x = SCREEN_WIDTH / 2;
    let mut player_y = SCREEN_HEIGHT /2 ;

    //create object representing player
    let player = Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', WHITE);

    //create an NPC
    let npc  = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', YELLOW);

    //the list of objects with those two
    let mut objects = [player, npc];

    let game = Game {
        map: make_map(),
    };

    while !tcod.root.window_closed() {
	tcod.con.set_default_foreground(WHITE);
	for object in &objects {
	    object.draw(&mut tcod.con);
	}
		
        tcod.con.clear();

        render_all(& mut tcod, &game, &objects);
	tcod.root.flush();
	tcod.root.wait_for_keypress(true);

        let player = &mut objects[0];
	let exit = handle_keys(&mut tcod, player);
	if exit {
	    break;
	}
    }
    
}
