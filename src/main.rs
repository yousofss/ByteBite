use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style::Print,
    terminal::{self, disable_raw_mode, enable_raw_mode, size},
    ExecutableCommand, QueueableCommand,
};
use rand::Rng;
use std::io::{stdout, Write};

#[derive(Clone)]
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

struct Snake {
    body: Vec<(u16, u16)>,
    direction: Direction,
}

impl Snake {
    fn move_snake(&mut self, new_direction: Direction) {
        // Update the snake's direction, but prevent 180-degree turns
        self.direction = match (&self.direction, &new_direction) {
            (Direction::UP, Direction::DOWN)
            | (Direction::DOWN, Direction::UP)
            | (Direction::LEFT, Direction::RIGHT)
            | (Direction::RIGHT, Direction::LEFT) => self.direction.clone(),
            _ => new_direction,
        };

        // Calculate the new head position
        let (head_x, head_y) = self.body[0];
        let new_head = match self.direction {
            Direction::UP => (head_x, head_y.saturating_sub(1)),
            Direction::DOWN => (head_x, head_y.saturating_add(1)),
            Direction::LEFT => (head_x.saturating_sub(1), head_y),
            Direction::RIGHT => (head_x.saturating_add(1), head_y),
        };

        // Move the body
        for i in (1..self.body.len()).rev() {
            self.body[i] = self.body[i - 1];
        }

        // Update the head
        self.body[0] = new_head;
    }

    fn is_self_collision(&self) -> bool {
        let head = self.body[0];
        self.body.iter().skip(1).any(|&segment| segment == head)
    }
}

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    let (x, y) = size().unwrap();

    let mut snake = Snake {
        body: vec![
            (x / 2, y / 2),
            (x / 2, (y / 2) + 1),
            (x / 2, (y / 2) + 2),
            (x / 2, (y / 2) + 3),
            (x / 2, (y / 2) + 4),
        ],
        direction: Direction::UP,
    };

    let mut rng = rand::thread_rng();
    let mut food = (rng.gen_range(0..x), rng.gen_range(0..y));

    loop {
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        stdout.queue(cursor::Hide)?;

        // Draw the snake
        for (i, &(x, y)) in snake.body.iter().enumerate() {
            stdout
                .queue(cursor::MoveTo(x, y))?
                .queue(Print(if i == 0 { "O" } else { "o" }))?;
        }

        // Draw the food
        stdout
            .queue(cursor::MoveTo(food.0, food.1))?
            .queue(Print("F"))?
            .flush()?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') => {
                        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
                        stdout.queue(Print("Quitting..."))?;
                        break;
                    }
                    KeyCode::Up => snake.move_snake(Direction::UP),
                    KeyCode::Down => snake.move_snake(Direction::DOWN),
                    KeyCode::Left => snake.move_snake(Direction::LEFT),
                    KeyCode::Right => snake.move_snake(Direction::RIGHT),
                    _ => continue,
                }
            }
        } else {
            snake.move_snake(snake.direction.clone());
        }

        // Check if snake ate the food
        if snake.body[0] == food {
            // Generate new food
            food = (rng.gen_range(0..x), rng.gen_range(0..y));
            while snake.body.contains(&food) {
                food = (rng.gen_range(0..x), rng.gen_range(0..y));
            }

            // Grow the snake
            let last = *snake.body.last().unwrap();
            snake.body.push(last);
        }

        if snake.is_self_collision() {
            stdout
                .execute(cursor::MoveTo(x / 2 - 5, y / 2))?
                .execute(Print("Game Over!"))?;
            break;
        }
        if snake.body[0].0 > x || snake.body[0].1 > y {
            stdout
                .execute(cursor::MoveTo(x / 2 - 5, y / 2))?
                .execute(Print("Game Over!"))?;
            break;
        }
    }

    disable_raw_mode()?;
    stdout.queue(cursor::Show)?;
    Ok(())
}
