use tcod::colors::*;
use tcod::console::*;
use std::cmp;
use rand::Rng;

use tcod::map::{FovAlgorithm, Map as FovMap};

//Actual size of the window

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const LIMIT_FPS: i32 = 20; //20 FPS max

const FOV_ALG: FovAlgorithm = FovAlgorithm::Basic; //Default FOV Algo
const FOV_LIGHT_WALS: bool - true; //light walls or not
const TORCH_RADIUS: i32 = 10;


const COLOR_DARK_WALL: Color = Color {r: 0, g: 0, b: 100};
const COLOR_LIGHT_WALL: Color = Color {r: 130, g: 110, b: 50};
const COLOR_DARK_GROUND: Color = Color {r: 0, g: 50, b: 150,};
const COLOR_LIGHT_GROUND: Color = Color {r: 200, g: 180, b: 50};

//parameters for dungeon generator
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

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
            blocked: true,
	    block_sight: true,
        }
    }
}

struct Tcod {
    root: Root,
    con: Offscreen,
    fov: FovMap,
}

type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
}

/// A rectangle on the map, used to chatacterise a room.
#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
	        x1: x,
	        y1: y,
	        x2: x + w,
	        y2: y + h,
	    }
    }

    pub fn center(&self) -> (i32, i32) {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;
        (center_x, center_y)
    }

    pub fn intersects_with(&self, other: &Rect) -> bool {
        //returns true if this rectangle intersects with another
        (self.x1 <= other.x2)
            && (self.x2 >= other.x1)
            && (self.y1 <= other.y2)
            && (self.y2 >= other.y1)
    }
}


fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
   //horizontal tunnel. `min()` and `max()` are used in case `x1>x2`
   for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
       map[x as usize][y as usize] = Tile::empty();
   }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    // vertical tunnel
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
       map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_room(room: Rect, map: &mut Map) {
    //go through the tiles in the rectangle and make them passable
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
	    map[x as usize][y as usize] = Tile::empty();
	}
    }
}

fn make_map(player: &mut Object) -> Map {
    // fill map with blocked tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    let mut rooms = vec![];
    for _ in 0..MAX_ROOMS {
        // random width and height
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        // random position without going out of the boundaries of the map
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        let new_room = Rect::new(x, y, w, h);
        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !failed {
            /// this means there are no intersections

            // paint into map tiles
            create_room(new_room, &mut map);

            // center coordinates of the newroom, will be useful later
            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                // this is the first room, where the player starts at
                player.x = new_x;
                player.y = new_y;
            } else {
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                if rand::random() {
                    // Horizontal, then vertical
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    // Vertical, then horizontal
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }
            //append new room to list
            rooms.push(new_room);
        }
    }
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

#[derive(Debug)]
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
    pub fn move_by(&mut self, dx: i32, dy: i32, game: &Game) {
         if !game.map[(self.x + dx) as usize][(self.y + dy) as usize].blocked {
		self.x += dx;
	 	self.y += dy;
	}
    }

    ///set color and then draw the character that represents this object at its position
    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
	con.put_char(self.x, self.y, self.char, BackgroundFlag:: None);
    }
}

fn handle_keys(tcod: &mut Tcod, game: &Game, player: &mut Object) -> bool {
    use tcod::input::Key;  
    use tcod::input::KeyCode::*;

    //handle keys here

    let key = tcod.root.wait_for_keypress(true);
    match key {
	//movement keys
	Key { code: Up, .. } => player.move_by(0, -1, game),
	Key { code: Down, .. } => player.move_by(0, 1, game),
	Key { code: Left, .. } => player.move_by(-1, 0, game),
	Key { code: Right, .. } => player.move_by(1, 0, game),

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

    let mut tcod = Tcod { 
        root, 
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT)
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
    };

    //create object representing player
    let player = Object::new(25, 23, '@', WHITE);

    //create an NPC
    let npc  = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', YELLOW);

    //the list of objects with those two
    let mut objects = [player, npc];

    let game = Game {
        map: make_map(&mut objects[0]),
    };

    while !tcod.root.window_closed() {
    	for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                tcod.fov.set(
                    x,
                    y,
                    !game.map[x as usize][y as usize].block_sight,
                    !game.map[x as usize][y as usize].blacked,
                    );
            }
        }

        tcod.con.set_default_foreground(WHITE);
    	for object in &objects {
	        object.draw(&mut tcod.con);
	    }
		
        tcod.con.clear();

        render_all(& mut tcod, &game, &objects);
	tcod.root.flush();
	tcod.root.wait_for_keypress(true);

        let player = &mut objects[0];
	let exit = handle_keys(&mut tcod, &game, player);
	if exit {
	    break;
	}
    }
    
}
