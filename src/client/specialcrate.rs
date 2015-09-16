// Code by Johannes Schickling <schickling.j@gmail.com>,
// https://github.com/schickling/rust-examples/tree/master/snake-ncurses
// minor revisions accoring to changed rust-spedicifactions were made in
// order to successfully compile the code, game.rs and main.rs were
// merged into specialcrate.rs. Code was published under MIT license.

extern crate rand;

use self::rand::Rng;

pub enum GameError { Wall, Suicide }

#[derive(PartialEq, Copy)]
pub struct Vector {
    pub x: i32,
    pub y: i32,
}

impl Vector {
    fn next (&self, dir: Direction) -> Vector {
        let (dx, dy) = match dir {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };

        Vector {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    fn random (bounds: Vector) -> Vector {
        let mut rng = rand::thread_rng();
        Vector {
            x: rng.gen_range::<>(0, bounds.x),
            y: rng.gen_range::<>(0, bounds.y),
        }
    }
}

impl Clone for Vector {

    fn clone(&self) -> Self {
        Vector { x : self.x, y : self.y}
    }

    fn clone_from(&mut self, source: &Self) {
        self.x = source.x;
        self.y = source.y;
    }
}

pub struct Board {
    bounds: Vector,
    snake: Snake,
    bullet: Vector,
}

impl Board {

    pub fn new (bounds: Vector) -> Board {
        Board {
            bounds: bounds,
            snake: Snake::new(Vector { x: bounds.x / 2, y: bounds.y / 2 }),
            bullet: Vector::random(bounds),
        }
    }

    pub fn set_direction (&mut self, dir: Direction) {
        self.snake.direction = dir;
    }

    pub fn tick (&mut self) -> Result<(), GameError> {

        self.snake.step();

        if self.snake.eats_bullet(self.bullet) {
            self.snake.grow();
            self.bullet = Vector::random(self.bounds);
        }

        if self.snake.hits_wall(self.bounds) {
            Err(GameError::Wall)
        } else if self.snake.hits_itself() {
            Err(GameError::Suicide)
        } else {
            Ok(())
        }
    }

    pub fn get_snake_vectors (&self) -> &[Vector] {
        let ref v = self.snake.segments;
        &v[..]
    }

    pub fn get_bullet_vector (&self) -> &Vector {
        &self.bullet
    }
}

struct Snake {
    segments: Vec<Vector>,
    direction: Direction,
    popped_segment: Vector,
}

impl Snake {

    fn new (pos: Vector) -> Snake {
        Snake {
            segments: vec!(pos),
            direction: Direction::Up,
            popped_segment: Vector { x: 0, y: 0 }
        }
    }

    fn step (&mut self) {
        let new_head = self.segments[0].next(self.direction);
        self.segments.insert(0, new_head);
        self.popped_segment = self.segments.pop().unwrap();
    }

    fn hits_wall (&self, bounds: Vector) -> bool {
        let head = self.segments[0];
        head.x < 0 || head.x == bounds.x || head.y < 0 || head.y == bounds.y
    }

    fn hits_itself (&self) -> bool {
        self.segments.iter().skip(1).any(|s| *s == self.segments[0] )
    }

    fn grow (&mut self) {
        self.segments.push(self.popped_segment);
    }

    fn eats_bullet (&self, bullet: Vector) -> bool {
        self.segments[0] == bullet
    }
}

// This was the code in main.rs
extern crate ncurses;

use std::thread::sleep_ms as sleep;
use self::ncurses::*;

pub fn snake()
{
    initscr();
    cbreak(); // enable <Ctrl+C> to kill game
    noecho(); // don't show input
    keypad(stdscr, true); // make keys work
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    timeout(100); // tick speed

    let mut bounds = Vector { x: 0, y: 0 };
    getmaxyx(stdscr, &mut bounds.y, &mut bounds.x);

    let mut board = Board::new(bounds);

    let mut direction = Direction::Up;

    loop {
        erase();

        {
            let bullet = board.get_bullet_vector();
            draw_char(bullet, 'o');
        }

        {
            let segments = board.get_snake_vectors();
            for segment in segments.iter() {
                draw_char(segment, '#');
            }
        }

        direction = get_new_direction(direction);
        board.set_direction(direction);


        match board.tick() {
            Err(err) => {
                match err {
                    GameError::Wall => show_text("You hit the wall, stupid."),
                    GameError::Suicide => show_text("Damn it. Stop eating yourself."),
                }
                //let two_secs = Duration::new(2, 0);
                sleep(2000);
                break;
            },
            Ok(_) => (),
        };
    }
    endwin();
}

fn draw_char (pos: &Vector, c: char) {
    mvaddch(pos.y, pos.x, c as u64);
}

fn get_new_direction (prev_dir: Direction) -> Direction {
    match getch() {
        KEY_UP if prev_dir != Direction::Down => Direction::Up,
        KEY_DOWN if prev_dir != Direction::Up => Direction::Down,
        KEY_LEFT if prev_dir != Direction::Right => Direction::Left,
        KEY_RIGHT if prev_dir != Direction::Left => Direction::Right,
        _ => prev_dir,
    }
}

fn show_text (s: &str) {
    erase();
    addstr(s);
    refresh();
}

// Below is space invaders code

#[derive(PartialEq,Copy,Clone)]
pub enum Direction { Up, Down, Left, Right }

pub enum GameStatus { Win, Running, Dead }

pub struct Game {
    bounds: Vector,
    invaders: Vec<Invader>,
    player: Player,
    bullets: Vec<Bullet>,
}

impl Game {

    pub fn new (bounds: Vector) -> Game {

        let mut invaders = vec!();
        //for i in range(0i32, bounds.x / 3) {
        for i in 0i32..(bounds.x / 3) {
            invaders.push(Invader::new( Vector { x: 2 * i, y: 0 }));
        }

        Game {
            bounds: bounds,
            invaders: invaders,
            player: Player::new(Vector { x: bounds.x / 2, y: bounds.y - 1 }),
            bullets: vec!(),
        }
    }

    pub fn tick (&mut self) -> GameStatus {

        for invader in self.invaders.iter_mut() {
            match invader.give_chance_to_fire() {
                Some(bullet) => self.bullets.push(bullet),
                None => (),
            };
            invader.tick(self.bounds);
        }

        for bullet in self.bullets.iter_mut() {
            bullet.tick(self.bounds);

            self.invaders.retain(|i| !bullet.check_collision(i.position));

            if bullet.check_collision(self.player.position) {
                return GameStatus::Dead;
            }
        }

        if self.invaders.is_empty() {
            GameStatus::Win
        } else {
            GameStatus::Running
        }
    }

    pub fn shift (&mut self, dir: Direction) {
        self.player.shift(dir, self.bounds);
    }

    pub fn fire (&mut self) {
        self.bullets.push(self.player.fire());
    }

    pub fn get_player_vector (&self) -> &Vector {
        &self.player.position
    }

    pub fn get_invader_vectors (&self) -> Vec<Vector> {
        self.invaders.iter().map(|i| i.position).collect()
    }

    pub fn get_bullet_vectors (&self) -> Vec<Vector> {
        self.bullets.iter().map(|b| b.position).collect()
    }
}

struct Invader {
    position: Vector,
    direction: Direction,
}

impl Invader {

    fn new (pos: Vector) -> Invader {
        Invader {
            position: pos,
            direction: Direction::Left,
        }
    }

    fn tick (&mut self, bounds: Vector) {
        let x = &mut self.position.x;
        self.direction = match self.direction {
            Direction::Left if *x < 0 => Direction::Right,
            Direction::Right if *x == bounds.x => Direction::Left,
            _ => self.direction.clone()
        };
        match self.direction {
            Direction::Left => *x = *x - 1,
            Direction::Right => *x = *x + 1,
            _ => (),
        };
    }

    fn give_chance_to_fire (&self) -> Option<Bullet> {
        let mut rng = rand::thread_rng();
        let temp: f32 = rng.gen_range(0.0, 1.0);
        if temp > 0.996 {
            Some(Bullet::new(Vector { x: self.position.x,
                y: self.position.y + 1 }, Direction::Down))
        } else {
            None
        }
    }
}

struct Player {
    position: Vector,
}

impl Player {

    fn new (pos: Vector) -> Player {
        Player { position: pos }
    }

    fn shift (&mut self, dir: Direction, bounds: Vector) {
        let x = &mut self.position.x;
        match dir {
            Direction::Left if *x > 0 => *x = *x - 1,
            Direction::Right if *x < bounds.x - 1 => *x = *x + 1,
            _ => (),
        }
    }

    fn fire (&self) -> Bullet {
        Bullet::new(Vector { x: self.position.x, y: self.position.y - 1 }, Direction::Up)
    }
}

struct Bullet {
    position: Vector,
    direction: Direction,
}

impl Bullet {

    fn new (pos: Vector, dir: Direction) -> Bullet {
        Bullet {
            position: pos,
            direction: dir,
        }
    }

    fn tick (&mut self, bounds: Vector) {
        match self.direction {
            Direction::Up => self.position.y -= 1,
            Direction::Down => self.position.y += 1,
            _ => (),
        };
        if self.position.y < 0 || self.position.y == bounds.y {
            drop(self);
        }
    }

    fn check_collision (&self, other: Vector) -> bool {
        self.position == other
    }
}

// blow is code from main.rs
pub fn space_invaders()
{
    initscr();
    cbreak(); // enable <Ctrl+C> to kill game
    noecho(); // don't show input
    keypad(stdscr, true); // make keys work
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    timeout(1);

    let mut bounds = Vector { x: 0, y: 0};
    getmaxyx(stdscr, &mut bounds.y, &mut bounds.x);

    let mut game = Box::new(Game::new(bounds));

    // TODO rewrite async
    let mut tick_count = 0u8;

    loop {
        tick_count += 1;
        tick_count %= 30;

        erase();

        {
            let player = game.get_player_vector();
            draw_char(player, 'X');
        }

        {
            let invaders = game.get_invader_vectors();
            for invader in invaders.iter() {
                draw_char(invader, '#');
            }
        }

        {
            let bullets = game.get_bullet_vectors();
            for bullet in bullets.iter() {
                draw_char(bullet, '.');
            }
        }

        match getch() {
            KEY_LEFT => game.shift(Direction::Left),
            KEY_RIGHT => game.shift(Direction::Right),
            KEY_UP => game.fire(),
            _ => (),
        }

        if tick_count == 0 {
            match game.tick() {
                GameStatus::Dead => {
                    show_text("Puny human!");
                    sleep(2000);
                    break;
                },
                GameStatus::Win => {
                    show_text("Garglargl");
                    sleep(2000);
                    break;
                },
                GameStatus::Running => (),
            };
        }
    }

    endwin();
}
