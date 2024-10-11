use std::io::stdout;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style::Print,
    terminal::{self, disable_raw_mode, enable_raw_mode, size},
    ExecutableCommand, QueueableCommand,
};
enum Direction {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    NOWHERE,
}
impl Clone for Direction {
    fn clone(&self) -> Self {
        match self {
            Direction::UP => Direction::UP,
            Direction::DOWN => Direction::DOWN,
            Direction::LEFT => Direction::LEFT,
            Direction::RIGHT => Direction::RIGHT,
            Direction::NOWHERE => Direction::NOWHERE,
        }
    }
}
struct Snake {
    body: Vec<(u16, u16, Direction)>,
    is_alive: bool,
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
        let (head_x, head_y) = (self.body[0].0, self.body[0].1);
        let new_head = match self.direction {
            Direction::UP => (head_x, head_y.saturating_sub(1)),
            Direction::DOWN => (head_x, head_y.saturating_add(1)),
            Direction::LEFT => (head_x.saturating_sub(1), head_y),
            Direction::RIGHT => (head_x.saturating_add(1), head_y),
            Direction::NOWHERE => (head_x, head_y),
        };

        // Move the body
        for i in (1..self.body.len()).rev() {
            self.body[i] = (
                self.body[i - 1].0,
                self.body[i - 1].1,
                self.body[i - 1].2.clone(),
            );
        }

        // Update the head
        self.body[0] = (new_head.0, new_head.1, self.direction.clone());

        // Check for self-collision
        if self
            .body
            .iter()
            .skip(1)
            .any(|segment| segment.0 == new_head.0 && segment.1 == new_head.1)
        {
            self.is_alive = false;
        }
    }
}

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    let (x, y) = size().unwrap();

    let mut snake = Snake {
        body: vec![
            (x / 2, y / 2, Direction::UP),
            (x / 2, (y / 2) + 1, Direction::NOWHERE),
            (x / 2, (y / 2) + 2, Direction::NOWHERE),
            (x / 2, (y / 2) + 3, Direction::NOWHERE),
            (x / 2, (y / 2) + 4, Direction::NOWHERE),
        ],
        is_alive: true,
        direction: Direction::UP,
    };

    loop {
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        stdout
            .queue(cursor::Hide)?
            .execute(cursor::MoveTo(snake.body[0].0, snake.body[0].1))?
            .execute(Print("A"))?;

        for i in snake.body.iter_mut().skip(1) {
            stdout
                .execute(cursor::MoveTo(i.0, i.1))?
                .execute(Print("S"))?;
        }
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') => {
                        terminal::Clear(terminal::ClearType::All);
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
        }
        if !snake.is_alive {
            stdout.queue(Print("You lost!"))?;
            break;
        }
    }

    disable_raw_mode()?;
    stdout.queue(cursor::Show)?;
    Ok(())
}
