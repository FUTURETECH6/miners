use bracket_terminal::prelude::*;
use specs::prelude::*;

use components::{Player, Position, Renderable, Viewshed, Monster};
use map::{Direction, Map};
use visibility_system::VisibilitySystem;
use monster_ai_system::MonsterAI;

mod components;
mod map;
mod player;
mod util;
mod visibility_system;
mod monster_ai_system;

const GAME_WIDTH: usize = 60;
const GAME_HEIGHT: usize = 50;

#[derive(PartialEq, Copy, Clone)]
pub enum State {
    Paused,
    Running
}


pub struct Game {
    pub world: World,
    pub state: State
}

impl Game {
    fn run_systems(&mut self) {
        let mut visibility = VisibilitySystem{};
        visibility.run_now(&self.world);
        let mut monsters = MonsterAI{};
        monsters.run_now(&self.world);

        // Apply changes to World
        self.world.maintain();
    }
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        // Reset console for next render
        ctx.cls();

        // Turn based updating
        if self.state == State::Running {
            self.run_systems();
            self.state = State::Paused;
        } else {
            self.state = player::input(self, ctx);
        }

        // Render map
        let map = self.world.fetch::<Map>();
        map.render(ctx);

        // Render entities
        let positions = self.world.read_storage::<Position>();
        let renderables = self.world.read_storage::<Renderable>();

        for (position, entity) in (&positions, &renderables).join() {
            let idx = map.xy_idx(position.x, position.y);
            // TODO: change entity light levels
            if let Some(light_level) = map.light_levels[idx] {
                ctx.print_color(position.x, position.y, entity.fg, entity.bg, entity.glyph);
            }
        }

        // Render FPS
        ctx.print_centered(0, &format!("{} fps", ctx.fps as u32));
    }
}

// Options: Kjammer_16x16, Md_16x16, Yayo16x16, Zilk16x16
bracket_terminal::embedded_resource!(TILE_FONT, "../resources/Zilk_16x16.png");

fn main() -> BError {
    bracket_terminal::link_resource!(TILE_FONT, "resources/Zilk_16x16.png");
    let context = BTermBuilder::new()
        .with_tile_dimensions(16, 16)
        .with_dimensions(GAME_WIDTH, GAME_HEIGHT)
        .with_font("Zilk_16x16.png", 16, 16)
        .with_title("miners")
        .with_simple_console(GAME_WIDTH, GAME_HEIGHT, "Zilk_16x16.png")
        // .with_automatic_console_resize(true)
        .build()?;

    let mut game: Game = Game {
        world: World::new(),
        state: State::Running,
    };

    game.world.register::<Position>();
    game.world.register::<Renderable>();
    game.world.register::<Player>();
    game.world.register::<Viewshed>();
    game.world.register::<Monster>();

    let mut map = Map::new(GAME_WIDTH, GAME_HEIGHT);

    let max_rooms: usize = 20;
    let min_room_size: usize = 3;
    let max_room_size: usize = 15;

    map.generate_map_rooms_and_corridors(max_rooms, min_room_size, max_room_size);

    let (player_x, player_y) = map.rooms[0].center();

    // place monsters in center of each room
    // for room in map.rooms.iter().skip(1) {
    //     let (x, y) = room.center();
    //     game.world.create_entity()
    //         .with(Position { x, y })
    //         .with(Renderable {
    //             glyph: 'g',
    //             fg: RGB::named(RED),
    //             bg: RGB::named(BLACK),
    //         })
    //         .with(Viewshed { visible_tiles: vec![], light_levels: vec![], strength: 5, dirty: true })
    //         .with(Monster {})
    //         .build();
    // }

    game.world.insert(map);

    // Create player
    game.world.create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: '☺',
            fg: RGB::from_f32(0.0, 1.0, 1.0),
            bg: RGB::from_f32(0.2, 0.2, 0.2),
        })
        .with(Player {})
        .with(Viewshed { visible_tiles: vec![], light_levels: vec![], strength: 8, dirty: true })
        .build();

    // Call into bracket_terminal to run the main loop. This handles rendering, and calls back into State's tick function every cycle. The box is needed to work around lifetime handling.
    main_loop(context, game)
}
