use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    style::{Color, Colors, Print, ResetColor, SetColors, SetForegroundColor},
    terminal::{self, disable_raw_mode, enable_raw_mode, size},
    ExecutableCommand, QueueableCommand,
};
use rand::{rngs::ThreadRng, Rng};
use serde_json::{json, Value};
use std::{
    fs,
    io::{stdout, Stdout, Write},
    path::Path,
    process::exit,
};

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
    fn move_snake(&mut self, new_direction: Direction, no_wall: bool, max_x: u16, max_y: u16) {
        // Update the snake's direction, but prevent 180-degree turns
        self.direction = match (&self.direction, &new_direction) {
            (Direction::UP, Direction::DOWN)
            | (Direction::DOWN, Direction::UP)
            | (Direction::LEFT, Direction::RIGHT)
            | (Direction::RIGHT, Direction::LEFT) => self.direction.clone(),
            _ => new_direction,
        };

        // Calculate the new head position
        let new_head: (u16, u16);
        let (head_x, head_y) = self.body[0];
        if no_wall {
            new_head = match self.direction {
                Direction::UP => {
                    if head_y.saturating_sub(1) <= 0 {
                        (head_x, max_y - 2)
                    } else {
                        (head_x, head_y.saturating_sub(1))
                    }
                }
                Direction::DOWN => {
                    if head_y.saturating_add(1) >= max_y - 1 {
                        (head_x, 1)
                    } else {
                        (head_x, head_y.saturating_add(1))
                    }
                }
                Direction::LEFT => {
                    if head_x.saturating_sub(1) <= 0 {
                        (max_x - 2, head_y)
                    } else {
                        (head_x.saturating_sub(1), head_y)
                    }
                }
                Direction::RIGHT => {
                    if head_x.saturating_add(1) >= max_x - 1 {
                        (1, head_y)
                    } else {
                        (head_x.saturating_add(1), head_y)
                    }
                }
            };
        } else {
            new_head = match self.direction {
                Direction::UP => (head_x, head_y.saturating_sub(1)),
                Direction::DOWN => (head_x, head_y.saturating_add(1)),
                Direction::LEFT => (head_x.saturating_sub(1), head_y),
                Direction::RIGHT => (head_x.saturating_add(1), head_y),
            };
        }
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

fn options(mut stdout: &Stdout, height: u16, width: u16) -> std::io::Result<()> {
    loop {
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        stdout.queue(cursor::Hide)?;

        let content =
            fs::read_to_string("./settings.json").expect("Should have been able to read the file");

        let mut v: Value = serde_json::from_str(&content)?;

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

        // Write out the options
        let height3 = height / 3;
        let width2 = width / 2;
        for y in height3..height3 + 3 {
            stdout
                .queue(cursor::MoveTo(width2, y))?
                .queue(Print(format!("[{}]", y - height3)))?
                .queue(ResetColor)?
                .flush()?;
        }
        // Write options text
        stdout
            .queue(cursor::MoveTo(width2 + 4, height3))?
            .queue(Print(format!(
                "{}vim mode",
                if v["vim_mode"].as_bool().unwrap() {
                    "-"
                } else {
                    ""
                }
            )))?
            .queue(ResetColor)?
            .flush()?;

        stdout
            .queue(cursor::MoveTo(width2 + 4, height3 + 1))?
            .queue(Print(format!(
                "{}no wall mode",
                if v["no_wall_mode"].as_bool().unwrap() {
                    "-"
                } else {
                    ""
                }
            )))?
            .queue(ResetColor)?
            .flush()?;

        stdout
            .queue(cursor::MoveTo(width2 + 4, height3 + 2))?
            .queue(Print(format!("exit")))?
            .queue(ResetColor)?
            .flush()?;

        if event::poll(std::time::Duration::from_millis(150))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('0') => {
                        let temp: bool = v["vim_mode"].as_bool().unwrap();
                        v["vim_mode"] = Value::Bool(!temp);
                        fs::write("./settings.json", v.to_string())?;
                    }
                    KeyCode::Char('1') => {
                        let temp: bool = v["no_wall_mode"].as_bool().unwrap();
                        v["no_wall_mode"] = Value::Bool(!temp);
                        fs::write("./settings.json", v.to_string())?;
                    }
                    KeyCode::Char('2') => return Ok(()),
                    _ => continue,
                };
            }
        }
    }
}
fn welcome_screen(mut stdout: &Stdout, height: u16, width: u16) -> std::io::Result<()> {
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

        // Write out the options
        let height3 = height / 3;
        let width2 = width / 2;
        for y in height3..height3 + 3 {
            stdout
                .queue(cursor::MoveTo(width2, y))?
                .queue(Print(format!("[{}]", y - height3)))?
                .queue(ResetColor)?
                .flush()?;
        }

        // Write options text
        stdout
            .queue(cursor::MoveTo(width2 + 4, height3))?
            .queue(Print(format!("Start the game!")))?
            .queue(ResetColor)?
            .flush()?;

        stdout
            .queue(cursor::MoveTo(width2 + 4, height3 + 1))?
            .queue(Print(format!("Options")))?
            .queue(ResetColor)?
            .flush()?;

        stdout
            .queue(cursor::MoveTo(width2 + 4, height3 + 2))?
            .queue(Print(format!("Quit")))?
            .queue(ResetColor)?
            .flush()?;

        if event::poll(std::time::Duration::from_millis(150))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('0') => return Ok(()),
                    KeyCode::Char('1') => options(stdout, height, width)?,
                    KeyCode::Char('2') => {
                        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
                        stdout.queue(Print("Quitting..."))?;
                        stdout.flush()?;
                        disable_raw_mode()?;
                        stdout.queue(cursor::Show)?;
                        exit(0);
                    }
                    _ => continue,
                };
            }
        }
    }
}

fn game_play(
    mut stdout: &Stdout,
    height: u16,
    width: u16,
    mut snake: Snake,
    mut rng: ThreadRng,
    mut food: (u16, u16),
    vim_mode: bool,
    no_wall: bool,
) -> std::io::Result<()> {
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

        // Show statistics
        stdout
            .queue(cursor::MoveTo(0, 0))?
            .queue(SetForegroundColor(Color::Red))?
            .queue(Print(format!("Score: {}", snake.body.iter().count() - 5)))?
            .queue(ResetColor)?;

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
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Up => snake.move_snake(Direction::UP, no_wall, width, height),
                    KeyCode::Down => snake.move_snake(Direction::DOWN, no_wall, width, height),
                    KeyCode::Left => snake.move_snake(Direction::LEFT, no_wall, width, height),
                    KeyCode::Right => snake.move_snake(Direction::RIGHT, no_wall, width, height),
                    // Vim mode
                    KeyCode::Char('k') => {
                        if vim_mode {
                            snake.move_snake(Direction::UP, no_wall, width, height)
                        } else {
                            // fix the repeating button issue that prevent snake from moving
                        }
                    }
                    KeyCode::Char('j') => {
                        if vim_mode {
                            snake.move_snake(Direction::DOWN, no_wall, width, height)
                        }
                    }
                    KeyCode::Char('h') => {
                        if vim_mode {
                            snake.move_snake(Direction::LEFT, no_wall, width, height)
                        }
                    }
                    KeyCode::Char('l') => {
                        if vim_mode {
                            snake.move_snake(Direction::RIGHT, no_wall, width, height)
                        }
                    }
                    _ => continue,
                }
            }
        } else {
            snake.move_snake(snake.direction.clone(), no_wall, width, height);
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
            || (!no_wall
                && (snake.body[0].0 == 0
                    || snake.body[0].0 == width - 1
                    || snake.body[0].1 == 0
                    || snake.body[0].1 == height - 1))
        {
            stdout
                .execute(terminal::Clear(terminal::ClearType::All))?
                .execute(cursor::MoveTo(width / 2 - 5, height / 2))?
                .execute(Print("Game Over!"))?;
            break;
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    // check the settings file exist, if yes load it else create one!
    if !Path::new("./settings.json").exists() {
        let data = json!({
            "vim_mode": false,
            "no_wall_mode": false,
        });
        fs::write("./settings.json", data.to_string())?;
    }
    enable_raw_mode()?;

    loop {
        let stdout = stdout();
        let (mut width, mut height) = size().unwrap();
        width /= 4; // Divide by 4 instead of 2 because we're using double-width characters
        height /= 2;

        welcome_screen(&stdout, height, width)?;

        let snake = Snake {
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
        let food = (rng.gen_range(1..width - 1), rng.gen_range(1..height - 1));

        let content = fs::read_to_string("./settings.json")
            .expect("Should have been able to read settings.json");

        let v: Value = serde_json::from_str(&content)?;

        game_play(
            &stdout,
            height,
            width,
            snake,
            rng,
            food,
            v["vim_mode"].as_bool().unwrap(),
            v["no_wall_mode"].as_bool().unwrap(),
        )?;
    }
}
