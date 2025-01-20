use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style::{Color, Colors, Print, ResetColor, SetColors},
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
    let (mut width, mut height) = size().unwrap();
    width /= 4; // Divide by 4 instead of 2 because we're using double-width characters
    height /= 2;

    let mut snake = Snake {
        body: vec![
            (width / 2, height / 2),
            (width / 2, (height / 2) + 1),
            (width / 2, (height / 2) + 2),
            (width / 2, (height / 2) + 3),
            (width / 2, (height / 2) + 4),
        ],
        direction: Direction::UP,
    };

    let mut rng = rand::thread_rng();
    let mut food = (rng.gen_range(1..width - 1), rng.gen_range(1..height - 1));

    loop {
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        stdout.queue(cursor::Hide)?;

        // Draw the border
        for y in 0..height {
            for x in 0..width {
                if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                    stdout
                        .queue(cursor::MoveTo(x * 2, y))?
                        .queue(SetColors(Colors::new(Color::Green, Color::Green)))?
                        .queue(Print("██"))?
                        .queue(ResetColor)?;
                }
            }
        }

        // Draw the snake
        for (i, &(x, y)) in snake.body.iter().enumerate() {
            stdout
                .queue(cursor::MoveTo(x * 2, y))?
                .queue(SetColors(Colors::new(Color::Red, Color::Red)))?
                .queue(Print(if i == 0 { "██" } else { "██" }))?
                .queue(ResetColor)?;
        }

        // Draw the food
        stdout
            .queue(cursor::MoveTo(food.0 * 2, food.1))?
            .queue(SetColors(Colors::new(Color::Yellow, Color::Yellow)))?
            .queue(Print("██"))?
            .queue(ResetColor)?
            .flush()?;

        if event::poll(std::time::Duration::from_millis(150))? {
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
            food = (rng.gen_range(1..width - 1), rng.gen_range(1..height - 1));
            while snake.body.contains(&food) {
                food = (rng.gen_range(1..width - 1), rng.gen_range(1..height - 1));
            }

            // Grow the snake
            let last = *snake.body.last().unwrap();
            snake.body.push(last);
        }

        if snake.is_self_collision()
            || snake.body[0].0 == 0
            || snake.body[0].0 == width - 1
            || snake.body[0].1 == 0
            || snake.body[0].1 == height - 1
        {
            stdout
                .execute(cursor::MoveTo(width / 2 - 5, height / 2))?
                .execute(Print("Game Over!"))?;
            break;
        }
    }

    disable_raw_mode()?;
    stdout.queue(cursor::Show)?;
    Ok(())
}
