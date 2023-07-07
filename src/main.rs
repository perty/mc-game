use macroquad::prelude::*;

const MOVEMENT_SPEED: f32 = 200.0;
const RADIUS: f32 = 16.0;

struct Shape {
    size: f32,
    speed: f32,
    x: f32,
    y: f32,
    collided: bool,
}

#[macroquad::main("McGame")]
async fn main() {
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
    let mut game_over = false;
    loop {
        if !game_over {
            let delta_time = get_frame_time();

            //clear_background(DARKPURPLE);

            if is_key_down(KeyCode::Right) {
                circle.x += MOVEMENT_SPEED * delta_time;
            }
            if is_key_down(KeyCode::Left) {
                circle.x -= MOVEMENT_SPEED * delta_time;
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
            for square in squares.iter_mut() {
                for bullet in bullets.iter_mut() {
                    if bullet.collides_with(square) {
                        bullet.collided = true;
                        square.collided = true;
                    }
                }
            }
            // Remove squares and bullets below bottom of screen or hit.
            squares.retain(|square| square.y < screen_width() + square.size);
            squares.retain(|square| !square.collided);
            bullets.retain(|bullet| bullet.y > 0.0 - bullet.size / 2.0);
            bullets.retain(|bullet| !bullet.collided);

            // Draw bullets first so they are below other shapes.
            for bullet in &bullets {
                draw_circle(bullet.x, bullet.y, bullet.size / 2.0, RED);
            }
            draw_circle(circle.x, circle.y, RADIUS, YELLOW);
            for square in &squares {
                draw_rectangle(
                    square.x - square.size / 2.0,
                    square.y - square.size / 2.0,
                    square.size,
                    square.size,
                    GREEN,
                );
            }

            if squares.iter().any(|square| circle.collides_with(square)) {
                game_over = true;
            }
        }

        if game_over {
            if is_key_pressed(KeyCode::Space) {
                squares.clear();
                bullets.clear();
                circle.x = screen_width() / 2.0;
                circle.y = screen_height() / 2.0;
                game_over = false;
            } else {
                let text = "Game Over!";
                let text_dimensions = measure_text(text, None, 50, 1.0);
                draw_text(
                    text,
                    screen_width() / 2.0 - text_dimensions.width / 2.0,
                    screen_height() / 2.0,
                    50.0,
                    RED,
                );
            }
        }
        next_frame().await
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
