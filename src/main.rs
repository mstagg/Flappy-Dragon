use bracket_lib::prelude::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 33.33;
const TERMINAL_VEL: f32 = 2.0;
const GRAVITY: f32 = 0.2;
const FLAP_VEL: f32 = -2.0;
const OBSTACLE_VEL: f32 = -1.0;

fn main() {
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()
        .expect("Error creating BTerm");

    main_loop(context, State::new()).expect("Error while running game");
}

struct State {
    player: Player,
    obstacle: Obstacle,
    frame_time: f32,
    mode: GameMode,
    score: i32,
}

impl State {
    fn new() -> Self {
        Self {
            mode: GameMode::Menu,
            frame_time: 0.0,
            obstacle: Obstacle::new(SCREEN_WIDTH as f32, 0),
            player: Player::new(5.0, 25.0),
            score: 0,
        }
    }

    fn main_menu_loop(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flappy Dragon!");
        ctx.print_centered(6, "Press [Space] to start.");
        ctx.print_centered(7, "Press [Esc] to quit.");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Space => self.reset(),
                VirtualKeyCode::Escape => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn playing_loop(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);

        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.player.apply_gravity();
            self.player.move_velocity();
            self.obstacle.move_velocity();

            if self.obstacle.x <= 0.0 {
                self.score += 1;
                self.obstacle = Obstacle::new(SCREEN_WIDTH as f32, self.score);
            }

            self.frame_time = 0.0;
        }

        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.apply_flap();
        }

        self.player.render(ctx);
        self.obstacle.render(ctx);
        ctx.print(0, 0, "Press [Space] to flap.");
        ctx.print(0, 1, &format!("Score: {}", self.score));

        if self.player.y > SCREEN_HEIGHT as f32 || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::GameOver;
        }
    }

    fn game_over_loop(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You Died!");
        ctx.print_centered(6, &format!("Score: {}", self.score));
        ctx.print_centered(7, "Press [Space] to play again.");
        ctx.print_centered(8, "Press [Esc] to quit.");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Space => self.reset(),
                VirtualKeyCode::Escape => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn reset(&mut self) {
        self.player = Player::new(5.0, 25.0);
        self.obstacle = Obstacle::new(SCREEN_WIDTH as f32, 0);
        self.frame_time = 0.0;
        self.score = 0;
        self.mode = GameMode::Playing;
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu_loop(ctx),
            GameMode::Playing => self.playing_loop(ctx),
            GameMode::GameOver => self.game_over_loop(ctx),
        }
    }
}

enum GameMode {
    Menu,
    Playing,
    GameOver,
}

struct Player {
    x: f32,
    y: f32,
    velocity: f32,
}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            velocity: 0.0,
        }
    }

    fn render(&self, ctx: &mut BTerm) {
        ctx.set(
            self.x as i32,
            self.y as i32,
            RGB::named(YELLOW),
            RGB::named(BLACK),
            to_cp437('@'),
        );
    }

    fn set_velocity(&mut self, vel: f32) {
        if vel > TERMINAL_VEL {
            self.velocity = TERMINAL_VEL;
        } else {
            self.velocity = vel;
        }
    }

    fn apply_gravity(&mut self) {
        self.set_velocity(self.velocity + GRAVITY);
    }

    fn apply_flap(&mut self) {
        self.set_velocity(FLAP_VEL);
    }

    fn move_velocity(&mut self) {
        let new_y = self.y + self.velocity;
        if new_y < 0.0 {
            self.y = 0.0;
        } else {
            self.y += self.velocity;
        }
    }
}

struct Obstacle {
    x: f32,
    gap_y: i32,
    size: i32,
    score: i32,
}

impl Obstacle {
    fn new(x: f32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(10, 40),
            size: i32::max(2, 20 - score),
            score,
        }
    }

    fn render(&self, ctx: &mut BTerm) {
        let half_size = self.size / 2;

        for y in 0..self.gap_y - half_size {
            ctx.set(self.x as i32, y, RED, BLACK, to_cp437('|'));
        }

        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(self.x as i32, y, RED, BLACK, to_cp437('|'));
        }
    }

    fn move_velocity(&mut self) {
        self.x += OBSTACLE_VEL - (self.score as f32 * 0.25);
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x as i32 == self.x as i32;
        let is_above_gap = player.y < (self.gap_y - half_size) as f32;
        let is_below_gap = player.y > (self.gap_y + half_size) as f32;
        does_x_match && (is_above_gap || is_below_gap)
    }
}
