use macroquad::audio;
use macroquad::prelude::*;
use std::f32::consts::PI;
use std::fmt::{Display, Formatter};

#[macroquad::main(window_conf)]
async fn main() {
    show_mouse(false);
    rand::srand(miniquad::date::now() as _);

    let hit_sound = audio::load_sound("resources/cannon_hit.ogg").await.unwrap();
    let turret_fire_sound = audio::load_sound("resources/rlauncher.ogg").await.unwrap();
    let explosion_sound = audio::load_sound("resources/explosion.ogg").await.unwrap();

    let mut game = Game::new(0);
    let buildings = create_buildings();
    let mut mini_gunner = Turret::new();
    let stages = load_stages();
    clear_background(Color::default());

    loop {
        match game.state {
            State::Main => {
                draw_main_menu(&stages[0]);
                if is_key_pressed(KeyCode::Space) {
                    game = init_game(stages[0]);
                } else if is_key_pressed(KeyCode::Escape) {
                    break;
                }
            }
            State::Playing(stage) => {
                if is_key_pressed(KeyCode::Escape) {
                    break;
                }
                if game.score.city_health == 0 {
                    game.state = State::Dead;
                }
                if game.score.total_hit == stage.total_missile_count {
                    game.state = State::Win;
                }

                draw(&buildings);
                draw_cursor();
                mini_gunner.draw();
                game.draw();

                if is_mouse_button_pressed(MouseButton::Left)
                    && game.bullets.len() < stage.max_bullet_count
                    && mini_gunner.is_fire_suitable()
                {
                    audio::play_sound_once(turret_fire_sound);
                    let bullet = Bullet::spawn(mini_gunner.muzzle_point);
                    game.bullets.push(bullet);
                }

                for e in game.explosions.iter_mut() {
                    for m in game.missiles.iter_mut() {
                        let nearest_point = Vec2::new(
                            get_max(
                                m.position.x,
                                get_min(m.position.x + MISSILE_LENGTH, e.location.x),
                            ),
                            get_max(
                                m.position.y,
                                get_min(m.position.y + MISSILE_LENGTH, e.location.y),
                            ),
                        );
                        let distance = Vec2::new(
                            e.location.x - nearest_point.x,
                            e.location.y - nearest_point.y,
                        );
                        if distance.length() <= e.radius {
                            //e.is_alive = false;
                            audio::play_sound_once(explosion_sound);
                            game.score.total_hit += 1;
                            game.score.total_point += 10;
                            m.is_alive = false;
                            continue;
                        }
                    }
                }

                for b in game.bullets.iter_mut() {
                    if b.target.distance(b.position) < 1. {
                        b.is_alive = false;
                        let expl = Explosion::spawn(b.target);
                        game.explosions.push(expl);
                    }
                    b.position += b.velocity * BULLET_SPEED_FACTOR;
                    b.draw();
                }

                for e in game.explosions.iter_mut() {
                    e.draw();
                    if e.life_time == 0 {
                        e.is_alive = false;
                    } else {
                        e.radius += EXPLOSION_RADIUS_RATE;
                        e.life_time -= 1;
                    }
                }

                for m in game.missiles.iter_mut() {
                    if m.lift_off_time == 0 {
                        m.position += m.velocity * stage.missile_speed_factor;
                        m.draw();

                        if m.position.y > screen_height() - CITY_HEIGHT {
                            m.is_alive = false;
                            game.score.city_health -= PENALTY_VALUE;
                            audio::play_sound_once(hit_sound);
                        }
                    } else {
                        m.lift_off_time -= 1;
                    }
                }

                game.explosions.retain(|e| e.is_alive);
                game.bullets.retain(|b| b.is_alive);
                game.missiles.retain(|m| m.is_alive);

                if game.missiles.len() <= stage.max_missile_count as usize {
                    let mut new_missiles =
                        create_missiles(stage.max_missile_count - game.missiles.len() as i32);
                    game.missiles.append(&mut new_missiles);
                }
            }
            State::Dead => {
                // println!("Commander! City has fatal damage.");
                draw_dead_menu(&game);
                if is_key_pressed(KeyCode::Space) {
                    game = init_game(stages[0]);
                } else if is_key_pressed(KeyCode::Escape) {
                    game.state = State::Main;
                }
            }
            State::Win => {
                draw_win_menu(&game);

                if game.current_stage == stages.len() {
                    game.state = State::End;
                }
                if is_key_pressed(KeyCode::Space) {
                    game.current_stage += 1;
                    game = init_game(stages[game.current_stage]);
                } else if is_key_pressed(KeyCode::Escape) {
                    game.state = State::Main;
                }
            }
            State::End => {
                draw_end_menu(&game);
                if is_key_pressed(KeyCode::Enter) {
                    println!("Developer Burak Selim Şenyurt");
                } else if is_key_pressed(KeyCode::Escape) {
                    game.state = State::Main;
                }
            }
        }

        next_frame().await
    }
}
pub enum State {
    Main,
    Playing(Stage),
    Dead,
    Win,
    End,
}

pub struct Game {
    pub score: Score,
    pub state: State,
    pub missiles: Vec<Missile>,
    pub bullets: Vec<Bullet>,
    pub explosions: Vec<Explosion>,
    pub current_stage: usize,
}

impl Game {
    pub fn new(current_stage: usize) -> Self {
        clear_background(BLACK);
        Game {
            score: Score::default(),
            state: State::Main,
            missiles: Vec::new(),
            bullets: Vec::new(),
            explosions: Vec::new(),
            current_stage,
        }
    }
    pub fn draw(&self) {
        let text = format!("{} STG {}", self.score, get_stage_name(self.current_stage));
        let size = measure_text(text.as_str(), None, 20, 1.);
        draw_text(
            text.as_str(),
            screen_width() * 0.5 - size.width * 0.5,
            screen_height() - size.height + 10.,
            20.,
            RED,
        );
    }
}

fn init_game(stage: Stage) -> Game {
    let mut game = Game::new(stage.level);
    game.state = State::Playing(stage);
    game.missiles = create_missiles(stage.max_missile_count);
    game
}

pub struct Score {
    pub total_hit: i32,
    pub total_point: i32,
    pub total_missed: i32,
    pub city_health: i32,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            total_point: 0,
            total_hit: 0,
            total_missed: 0,
            city_health: MAX_CITY_HEALTH,
        }
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "City Health {}, Player Hit/Missed({}/{}) Point {}",
            self.city_health, self.total_hit, self.total_missed, self.total_point
        )
    }
}

#[derive(Clone, Copy)]
pub struct Stage {
    pub level: usize,
    pub max_missile_count: i32,
    pub total_missile_count: i32,
    pub missile_speed_factor: f32,
    pub max_bullet_count: usize,
}

impl Stage {
    pub fn new(
        level: usize,
        max_missile_count: i32,
        total_missile_count: i32,
        missile_speed_factor: f32,
        max_bullet_count: usize,
    ) -> Self {
        Self {
            level,
            max_missile_count,
            total_missile_count,
            missile_speed_factor,
            max_bullet_count,
        }
    }
}

pub fn get_stage_name(level: usize) -> String {
    match level {
        0 => "Rookie".to_string(),
        1 => "Specialist".to_string(),
        2 => "Veteran".to_string(),
        _ => "".to_string(),
    }
}

pub fn load_stages() -> Vec<Stage> {
    let rookie = Stage::new(
        0,
        MAX_MISSILE_COUNT_SAME_TIME,
        10,
        MISSILE_SPEED_FACTOR,
        MAX_BULLET_ON_GAME,
    );
    let specialist = Stage::new(
        1,
        MAX_MISSILE_COUNT_SAME_TIME + 5,
        15,
        MISSILE_SPEED_FACTOR * 1.5,
        MAX_BULLET_ON_GAME + 1,
    );
    let veteran = Stage::new(
        2,
        MAX_MISSILE_COUNT_SAME_TIME,
        15,
        MISSILE_SPEED_FACTOR * 3.,
        MAX_BULLET_ON_GAME + 2,
    );
    vec![rookie, specialist, veteran]
}

pub fn draw_main_menu(stage: &Stage) {
    let lines = vec![
        "Wellcome Commander".to_string(),
        "".to_string(),
        format!("Stage {}", get_stage_name(stage.level)),
        "Are you ready! Press SPACE to start".to_string(),
        "Press ESC to exit".to_string(),
    ];
    draw_menu(&lines);
}

pub fn draw_dead_menu(game: &Game) {
    let lines = vec![
        "City down. Sorry Commander.".to_string(),
        "".to_string(),
        game.score.to_string(),
        "Try again? Press SPACE".to_string(),
        "Press ESC to exit".to_string(),
    ];

    draw_menu(&lines);
}

pub fn draw_win_menu(game: &Game) {
    let lines = vec![
        "Yow win Commander.".to_string(),
        "".to_string(),
        game.score.to_string(),
        "Are you ready to next stage? Press SPACE".to_string(),
        "Press ESC to exit".to_string(),
    ];

    draw_menu(&lines);
}

pub fn draw_end_menu(game: &Game) {
    let lines = vec![
        "Congratulaions Commander".to_string(),
        "".to_string(),
        "You finished the game. City is safe.".to_string(),
        game.score.to_string(),
        "Press ENTER for Credits".to_string(),
        "Press ESC to exit".to_string(),
    ];

    draw_menu(&lines);
}

fn draw_menu(lines: &[String]) {
    for (i, line) in lines.iter().enumerate() {
        let size = measure_text(line, None, 32, 1.);
        draw_text(
            line,
            screen_width() * 0.5 - size.width * 0.5,
            screen_height() * 0.5 - size.height + (36. * i as f32),
            32.,
            RED,
        );
    }
}

pub fn draw_cursor() {
    let (x, y) = mouse_position();
    draw_line(x - CURSOR_LENGTH, y, x + CURSOR_LENGTH, y, 1., RED);
    draw_line(x, y - CURSOR_LENGTH, x, y + CURSOR_LENGTH, 1., RED);

    draw_text(
        format!("{:?}", mouse_position()).as_str(),
        0.,
        screen_height() - 5.,
        20.,
        RED,
    );
}

pub fn window_conf() -> Conf {
    Conf {
        window_title: "Missile Command".to_owned(),
        fullscreen: false,
        window_width: WINDOW_WITH,
        window_height: WINDOW_HEIGHT,
        window_resizable: false,
        ..Default::default()
    }
}

pub fn get_max(v1: f32, v2: f32) -> f32 {
    if v1 > v2 {
        return v1;
    }
    v2
}

pub fn get_min(v1: f32, v2: f32) -> f32 {
    if v1 < v2 {
        return v1;
    }
    v2
}

pub struct Bullet {
    pub position: Vec2,
    pub velocity: Vec2,
    pub target: Vec2,
    pub is_alive: bool,
}

impl Bullet {
    pub fn spawn(location: Vec2) -> Self {
        let mp = mouse_position();
        let dv = Vec2::new(mp.0 - location.x, mp.1 - location.y);
        Self {
            position: location,
            target: Vec2::new(mp.0, mp.1),
            velocity: dv.normalize(),
            is_alive: true,
        }
    }

    pub fn draw(&self) {
        let p = self.position - BULLET_WIDTH * 0.5;
        draw_rectangle(p.x, p.y, BULLET_WIDTH, BULLET_WIDTH, SKYBLUE);
    }
}

pub struct City {
    position: Vec2,
    size: Vec2,
}

impl City {
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self { position, size }
    }
    pub fn draw(&self) {
        draw_rectangle(
            self.position.x,
            self.position.y,
            self.size.x,
            self.size.y,
            LIGHTGRAY,
        );
    }
}

pub fn create_buildings() -> Vec<City> {
    let heights = [30., 25., 60., 40., 45., 55., 70., 20.];
    let mut buildings: Vec<City> = Vec::new();
    let mut position = Vec2::new(0., screen_height());
    let build_count = screen_width() / BASE_LENGTH;
    for _ in 0..build_count as i32 {
        let r = rand::gen_range(0, heights.len());
        position.y = screen_height() - heights[r];
        let city = City::new(position, Vec2::new(BASE_LENGTH, heights[r]));
        buildings.push(city);
        position.x += BASE_LENGTH;
    }
    buildings
}

pub fn draw(buildings: &Vec<City>) {
    for b in buildings {
        b.draw();
    }
}

pub struct Explosion {
    pub location: Vec2,
    pub life_time: usize,
    pub radius: f32,
    pub is_alive: bool,
}

impl Explosion {
    pub fn spawn(location: Vec2) -> Self {
        Self {
            location,
            life_time: EXPLOSION_LIFE_TIME,
            radius: 0.,
            is_alive: true,
        }
    }

    pub fn draw(&self) {
        draw_circle_lines(
            self.location.x,
            self.location.y,
            self.radius,
            EXPLOSION_THICKNESS,
            SKYBLUE,
        );
    }
}

pub struct Missile {
    start_position: Vec2,
    pub position: Vec2,
    pub velocity: Vec2,
    angle: f32,
    pub is_alive: bool,
    pub lift_off_time: i32,
}

impl Missile {
    pub fn spawn() -> Self {
        let x = rand::gen_range(
            screen_width() * 0.25,
            screen_width() - screen_width() * 0.25,
        );

        let c = Vec2::new(0., screen_height());
        let a = Vec2::new(0. - x, screen_height());
        let b = Vec2::new(screen_width() - x, screen_height());
        let mut left_angle = (a.dot(c) / (a.length() * c.length())).acos();
        let mut right_angle = (b.dot(c) / (b.length() * c.length())).acos();
        left_angle += PI / 2.;
        right_angle = PI / 2. - right_angle;
        let angle: f32 = rand::gen_range(right_angle, left_angle);

        Self {
            start_position: Vec2::new(x, 0.),
            position: Vec2::new(x, 0.),
            velocity: Vec2::new(angle.cos(), angle.sin()),
            angle,
            is_alive: true,
            lift_off_time: rand::gen_range(get_fps() as i32, get_fps() as i32 + MAX_LIFT_OFF_TIME),
        }
    }

    pub fn draw(&self) {
        draw_line(
            self.start_position.x,
            self.start_position.y,
            self.position.x,
            self.position.y,
            TRACE_TICKNESS,
            WHITE,
        );
        draw_rectangle(
            self.position.x,
            self.position.y,
            MISSILE_LENGTH,
            MISSILE_LENGTH,
            RED,
        );
    }
}

impl Display for Missile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pos: {}, Dir: {}, Angle:{} rad,{} deg, lift off {}",
            self.position,
            self.velocity,
            self.angle,
            self.angle.to_degrees(),
            self.lift_off_time
        )
    }
}

pub fn create_missiles(quantity: i32) -> Vec<Missile> {
    let mut missiles = Vec::new();
    for _ in 0..quantity {
        let missile = Missile::spawn();
        missiles.push(missile);
    }
    missiles
}

pub struct Turret {
    location: Vec2,
    unit_vector: Vec2,
    pub muzzle_point: Vec2,
}

impl Turret {
    pub fn new() -> Self {
        Self {
            location: Vec2::new(screen_width() * 0.5, screen_height() - CITY_HEIGHT),
            unit_vector: Vec2::default(),
            muzzle_point: Vec2::default(),
        }
    }

    pub fn is_fire_suitable(&self) -> bool {
        let f = Vec2::new(1., 0.);
        let angle = self.unit_vector.angle_between(f);
        if angle > PI / 6. && angle < 5. * PI / 6. {
            return true;
        }
        false
    }

    pub fn draw(&mut self) {
        let m = mouse_position();
        let v = Vec2::new(m.0 - self.location.x, m.1 - self.location.y);
        let mut unit_v = v.normalize();
        self.unit_vector = unit_v;
        unit_v = find_right_angle(unit_v);
        let add_v = unit_v * TURRET_MULTIPLIER;
        let mp = Vec2::new(self.location.x + add_v.x, self.location.y - add_v.y);
        self.muzzle_point = mp;
        draw_line(
            self.location.x,
            self.location.y,
            mp.x,
            mp.y,
            TURRET_THICKNESS,
            DARKGREEN,
        );
    }
}

pub fn find_right_angle(unit_vector: Vec2) -> Vec2 {
    let f = Vec2::new(1., 0.);
    let angle = unit_vector.angle_between(f);
    //println!("Angle {} , {}",angle,angle.to_degrees());
    if angle > 0. && angle <= PI / 6. {
        return Vec2::new((PI / 6.).cos(), (PI / 6.).sin());
    } else if (((5. * PI) / 6.)..=2. * PI).contains(&angle) {
        return Vec2::new((5. * PI / 6.).cos(), (5. * PI / 6.).sin());
    } else if angle < 0. {
        return Vec2::new((PI / 2.).cos(), (PI / 2.).sin());
    }
    Vec2::new(angle.cos(), angle.sin())
}

const MISSILE_LENGTH: f32 = 4.;
const TRACE_TICKNESS: f32 = 3.;
const CURSOR_LENGTH: f32 = 10.;
const MAX_MISSILE_COUNT_SAME_TIME: i32 = 5;
const MISSILE_SPEED_FACTOR: f32 = 0.35;
const BASE_LENGTH: f32 = 32.;
const WINDOW_WITH: i32 = 1024;
const WINDOW_HEIGHT: i32 = 768;
const MAX_LIFT_OFF_TIME: i32 = 500;
const MAX_CITY_HEALTH: i32 = 1000;
const CITY_HEIGHT: f32 = 100.;
const PENALTY_VALUE: i32 = 100;
const TURRET_MULTIPLIER: f32 = 30.;
const TURRET_THICKNESS: f32 = 3.;
const BULLET_WIDTH: f32 = 4.;
const BULLET_SPEED_FACTOR: f32 = 2.;
const MAX_BULLET_ON_GAME: usize = 3;
const EXPLOSION_LIFE_TIME: usize = 100;
const EXPLOSION_THICKNESS: f32 = 1.;
const EXPLOSION_RADIUS_RATE: f32 = 0.25;