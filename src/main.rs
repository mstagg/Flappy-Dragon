use bracket_lib::prelude::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 20.0;
const TERMINAL_VEL: f32 = 2.0;
const GRAVITY: f32 = 0.2;
const FLAP_VEL: f32 = -2.0;
const OBSTACLE_VEL: f32 = -1.0;

fn main() {
    let context = BTermBuilder::simple80x50()
        .with_fancy_console(SCREEN_WIDTH, SCREEN_HEIGHT, "terminal8x8.png")
        .with_title("Flappy Dragon")
        .with_vsync(false)
        .build()
        .expect("Error creating BTerm");

    main_loop(context, State::new()).expect("Error while running game");
}

struct State {
    player: Player,
    obstacle: Obstacle,
    mode: GameMode,
    frame_time: f32,
    score: f32,
}

impl State {
    fn new() -> Self {
        Self {
            mode: GameMode::Menu,
            frame_time: 0.0,
            obstacle: Obstacle::new(SCREEN_WIDTH as f32, 0.0),
            player: Player::new(5.0, 25.0),
            score: 0.0,
        }
    }

    fn main_menu_loop(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(0);
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flappy Dragon!");
        ctx.print_centered(6, "Press [Space] to start.");
        ctx.print_centered(7, "Press [Esc] to quit.");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Space => self.reset(ctx),
                VirtualKeyCode::Escape => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn playing_loop(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(0);
        ctx.cls_bg(NAVY);

        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.player.apply_gravity();
            self.player.move_velocity();
            self.obstacle.check_collision_and_move(&self.player);

            if self.player.position.y > SCREEN_HEIGHT as f32 || self.obstacle.collided {
                self.mode = GameMode::GameOver;
            }

            if self.obstacle.x <= 0.0 {
                self.score += 1.0;
                self.obstacle = Obstacle::new(SCREEN_WIDTH as f32, self.score);
            }

            self.frame_time = 0.0;
        }

        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.apply_flap();
        }

        ctx.print(0, 0, "Press [Space] to flap.");
        ctx.print(0, 1, &format!("Score: {}", self.score));
        ctx.set_active_console(1);
        ctx.cls();
        self.player.render(ctx);
        self.obstacle.render(ctx);
    }

    fn game_over_loop(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(0);
        ctx.cls();
        ctx.print_centered(5, "You Died!");
        ctx.print_centered(6, &format!("Score: {}", self.score));
        ctx.print_centered(7, "Press [Space] to play again.");
        ctx.print_centered(8, "Press [Esc] to quit.");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Space => self.reset(ctx),
                VirtualKeyCode::Escape => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn reset(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(1);
        ctx.cls();
        self.player = Player::new(5.0, 25.0);
        self.obstacle = Obstacle::new(SCREEN_WIDTH as f32, 0.0);
        self.frame_time = 0.0;
        self.score = 0.0;
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
    position: PointF,
    velocity: f32,
}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Self {
            position: PointF::new(x, y),
            velocity: 0.0,
        }
    }

    fn render(&self, ctx: &mut BTerm) {
        ctx.set_fancy(
            self.position,
            1,
            Degrees::new(self.velocity * 10.0),
            PointF::new(2.0, 2.0),
            RGB::named(GREEN),
            RGB::named(NAVY),
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
        let new_y = self.position.y + self.velocity;
        if new_y < 0.0 {
            self.position.y = 0.0;
        } else {
            self.position.y += self.velocity;
        }
    }
}

struct Obstacle {
    x: f32,
    gap_y: f32,
    size: f32,
    score: f32,
    collided: bool,
}

impl Obstacle {
    fn new(x: f32, score: f32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range::<f32>(10.0, 40.0),
            size: f32::max(2.0, 20.0 - score),
            score,
            collided: false,
        }
    }

    fn render(&self, ctx: &mut BTerm) {
        let half_size = self.size / 2.0;

        for y in 0..f32::floor(self.gap_y - half_size) as i32 {
            ctx.set_fancy(
                PointF::new(self.x, y as f32),
                1,
                Degrees::new(0.0),
                PointF::new(2.0, 1.0),
                BLACK,
                GRAY,
                to_cp437('|'),
            );
        }

        for y in f32::floor(self.gap_y + half_size) as i32..=SCREEN_HEIGHT {
            ctx.set_fancy(
                PointF::new(self.x, y as f32),
                1,
                Degrees::new(0.0),
                PointF::new(2.0, 1.0),
                BLACK,
                GRAY,
                to_cp437('|'),
            );
        }
    }

    fn vel(&self) -> f32 {
        OBSTACLE_VEL - (self.score * 0.25)
    }

    fn move_velocity(&mut self) {
        self.x += self.vel();
    }

    fn check_collision_and_move(&mut self, player: &Player) {
        let half_size = self.size / 2.0;
        let will_cross_player =
            self.x >= player.position.x && self.x + self.vel() <= player.position.x;
        let player_above_gap = player.position.y < (self.gap_y - half_size);
        let player_below_gap = player.position.y > (self.gap_y + half_size);

        let will_collide = will_cross_player && (player_above_gap || player_below_gap);
        self.collided = will_collide;
        if !will_collide {
            self.move_velocity();
        }
    }
}
