use std::fs;

use macroquad::experimental::animation::{AnimatedSprite, Animation};
use macroquad::prelude::*;
use macroquad_particles::{ColorCurve, Emitter, EmitterConfig};

const FRAGMENT_SHADER: &str = include_str!("starfield-shader.glsl");

const VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying float iTime;

uniform mat4 Model;
uniform mat4 Projection;
uniform vec4 _Time;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    iTime = _Time.x;
}
";

const MOVEMENT_SPEED: f32 = 200.0;
const RADIUS: f32 = 16.0;

struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
    collided: bool,
}

enum GameState {
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

#[macroquad::main("McGame")]
async fn main() {
    set_pc_assets_folder("assets");
    let ship_texture: Texture2D = load_texture("ship.png").await.expect("Couldn't load file");
    ship_texture.set_filter(FilterMode::Nearest);
    let bullet_texture: Texture2D = load_texture("laser-bolts.png")
        .await
        .expect("Couldn't load file");
    bullet_texture.set_filter(FilterMode::Nearest);
    build_textures_atlas();
    let mut ship_sprite = AnimatedSprite::new(
        16,
        24,
        &[
            Animation {
                name: "idle".to_string(),
                row: 0,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "left".to_string(),
                row: 2,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "right".to_string(),
                row: 4,
                frames: 2,
                fps: 12,
            },
        ],
        true,
    );
    let mut bullet_sprite = AnimatedSprite::new(
        16,
        16,
        &[
            Animation {
                name: "bullet".to_string(),
                row: 0,
                frames: 2,
                fps: 12,
            },
            Animation {
                name: "bolt".to_string(),
                row: 1,
                frames: 2,
                fps: 12,
            },
        ],
        true,
    );
    bullet_sprite.set_animation(1);

    let mut score: u32 = 0;
    let mut high_score: u32 = fs::read_to_string("highscore.dat")
        .map_or(Ok(0), |i| i.parse::<u32>())
        .unwrap_or(0);
    rand::srand(miniquad::date::now() as u64);
    let mut squares = vec![];
    let mut circle = Shape {
        size: 32.0,
        speed: MOVEMENT_SPEED,
        x: screen_width() / 2.0,
        y: screen_height() / 2.0,
        collided: false,
    };
    let mut bullets: Vec<Shape> = vec![];
    let mut explosions: Vec<(Emitter, Vec2)> = vec![];
    let mut game_state = GameState::MainMenu;
    let mut reached_high_score = false;

    let mut direction_modifier = 0.0;
    let render_target = render_target(320, 150);
    render_target.texture.set_filter(FilterMode::Nearest);
    let material = load_material(
        VERTEX_SHADER,
        FRAGMENT_SHADER,
        MaterialParams {
            uniforms: vec![
                ("iResolution".to_owned(), UniformType::Float2),
                ("direction_modifier".to_owned(), UniformType::Float1),
            ],
            ..Default::default()
        },
    )
        .unwrap();

    loop {
        clear_background(BLACK);
        material.set_uniform("iResolution", (screen_width(), screen_height()));
        material.set_uniform("direction_modifier", direction_modifier);
        gl_use_material(material);
        draw_texture_ex(
            render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        gl_use_default_material();

        match game_state {
            GameState::MainMenu => {
                if is_key_pressed(KeyCode::Escape) {
                    std::process::exit(0);
                }
                if is_key_pressed(KeyCode::Space) {
                    squares.clear();
                    bullets.clear();
                    explosions.clear();
                    circle.x = screen_width() / 2.0;
                    circle.y = screen_height() / 2.0;
                    score = 0;
                    game_state = GameState::Playing;
                }
                show_message("McGame shooter", 0.0, RED);
                show_message("Press space to start, escape to quit", 1.0, WHITE);
            }
            GameState::Playing => {
                let delta_time = get_frame_time();

                draw_text(
                    format!("Poäng: {}", score).as_str(),
                    10.0,
                    35.0,
                    25.0,
                    WHITE,
                );
                let highscore_text = format!("High score: {}", high_score);
                let text_dimensions = measure_text(highscore_text.as_str(), None, 25, 1.0);
                draw_text(
                    highscore_text.as_str(),
                    screen_width() - text_dimensions.width - 10.0,
                    35.0,
                    25.0,
                    WHITE,
                );

                ship_sprite.set_animation(0);
                if is_key_down(KeyCode::Right) {
                    circle.x += MOVEMENT_SPEED * delta_time;
                    direction_modifier += 0.05 * delta_time;
                    ship_sprite.set_animation(2);
                }
                if is_key_down(KeyCode::Left) {
                    circle.x -= MOVEMENT_SPEED * delta_time;
                    direction_modifier -= 0.05 * delta_time;
                    ship_sprite.set_animation(1);
                }
                if is_key_down(KeyCode::Down) {
                    circle.y += MOVEMENT_SPEED * delta_time;
                }
                if is_key_down(KeyCode::Up) {
                    circle.y -= MOVEMENT_SPEED * delta_time;
                }
                if is_key_pressed(KeyCode::Space) {
                    bullets.push(Shape {
                        x: circle.x,
                        y: circle.y,
                        speed: circle.speed * 2.0,
                        size: 5.0,
                        collided: false,
                    });
                }
                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Paused;
                }
                circle.x = circle.x.min(screen_width() - RADIUS).max(RADIUS);
                circle.y = circle.y.min(screen_height() - RADIUS).max(RADIUS);

                // Generate a new square maybe
                if rand::gen_range(0, 99) >= 95 {
                    let size = rand::gen_range(16.0, 64.0);
                    squares.push(Shape {
                        size,
                        speed: rand::gen_range(50.0, 150.0),
                        x: rand::gen_range(size / 2.0, screen_width() - size / 2.0),
                        y: -size,
                        collided: false,
                    });
                }

                // Move squares
                for square in &mut squares {
                    square.y += square.speed * delta_time;
                }
                // Move bullets
                for bullet in &mut bullets {
                    bullet.y -= bullet.speed * delta_time;
                }
                ship_sprite.update();
                bullet_sprite.update();
                for square in squares.iter_mut() {
                    for bullet in bullets.iter_mut() {
                        if bullet.collides_with(square) {
                            bullet.collided = true;
                            square.collided = true;
                            score += square.size.round() as u32;
                            high_score = high_score.max(score);
                            explosions.push((
                                Emitter::new(EmitterConfig {
                                    amount: square.size.round() as u32 * 2,
                                    ..particle_explosion()
                                }),
                                vec2(square.x, square.y),
                            ));
                        }
                    }
                }
                // Remove squares and bullets below bottom of screen or hit.
                squares.retain(|square| square.y < screen_width() + square.size);
                squares.retain(|square| !square.collided);
                bullets.retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);
                bullets.retain(|bullet| !bullet.collided);
                // And the explosions
                explosions.retain(|(explosion, _)| explosion.config.emitting);

                // Draw bullets first so they are below other shapes.
                let bullet_frame = bullet_sprite.frame();
                for bullet in &bullets {
                    draw_texture_ex(
                        bullet_texture,
                        bullet.x - bullet.size / 2.0,
                        bullet.y - bullet.size / 2.0,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(bullet.size, bullet.size)),
                            source: Some(bullet_frame.source_rect),
                            ..Default::default()
                        },
                    )
                }
                // Draw ship
                let ship_frame = ship_sprite.frame();
                draw_texture_ex(
                    ship_texture,
                    circle.x - ship_frame.dest_size.x,
                    circle.y - ship_frame.dest_size.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(ship_frame.dest_size * 2.0),
                        source: Some(ship_frame.source_rect),
                        ..Default::default()
                    },
                );

                for square in &squares {
                    draw_rectangle(
                        square.x - square.size / 2.0,
                        square.y - square.size / 2.0,
                        square.size,
                        square.size,
                        GREEN,
                    );
                }
                // Draw explosions
                for (explosion, coords) in explosions.iter_mut() {
                    explosion.draw(*coords);
                }

                if squares.iter().any(|square| circle.collides_with(square)) {
                    game_state = GameState::GameOver;
                    if score == high_score {
                        fs::write("highscore.dat", high_score.to_string()).ok();
                        reached_high_score = true;
                    }
                }
            }
            GameState::Paused => {
                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::Playing;
                }
                show_message("Pause", 0.0, WHITE);
            }
            GameState::GameOver => {
                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::MainMenu;
                    reached_high_score = false;
                } else {
                    show_message("Game Over!", 0.0, RED);
                    if reached_high_score {
                        show_message("Congratulations! High score!", 1.0, GREEN);
                    } else {
                        show_message("Better luck next time!", 1.0, GREEN);
                    }
                    show_message("Press escape for main menu", 2.0, YELLOW);
                }
            }
        }
        next_frame().await
    }
}

fn show_message(text: &str, row: f32, color: Color) {
    let text_dimensions = measure_text(text, None, 50, 1.0);
    draw_text(
        text,
        screen_width() / 2.0 - text_dimensions.width / 2.0,
        screen_height() / 2.0 + text_dimensions.height * row * 1.40,
        50.0,
        color,
    );
}

fn particle_explosion() -> EmitterConfig {
    EmitterConfig {
        local_coords: false,
        one_shot: true,
        emitting: true,
        lifetime: 0.8,
        lifetime_randomness: 0.7,
        explosiveness: 0.75,
        initial_direction_spread: 2.0 * std::f32::consts::PI,
        initial_velocity: 300.0,
        initial_velocity_randomness: 0.8,
        size: 3.0,
        size_randomness: 0.3,
        colors_curve: ColorCurve {
            start: RED,
            mid: ORANGE,
            end: RED,
        },
        ..Default::default()
    }
}

impl Shape {
    fn collides_with(&self, other: &Self) -> bool {
        self.rect().overlaps(&other.rect())
    }

    fn rect(&self) -> Rect {
        Rect {
            x: self.x - self.size / 2.0,
            y: self.y - self.size / 2.0,
            w: self.size,
            h: self.size,
        }
    }
}
