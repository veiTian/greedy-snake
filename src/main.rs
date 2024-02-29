use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use rand::Rng;
use std::sync::{Arc, Mutex};

// 方向
enum Direction {
    Up,
    Down,
    Right,
    Left,
}

// 位置
struct Position {
    x: isize,
    y: isize,
}

// 蛇
struct Snake {
    direction: Direction,
    head: Position,
    body: Vec<Position>,
    has_eaten: bool,
}
impl Snake {
    fn new(x: isize, y: isize) -> Self {
        Self {
            direction: Direction::Right,
            head: Position { x, y },
            body: vec![Position { x: x - 1, y }],
            has_eaten: false,
        }
    }

    // 检测是否吃到食物
    fn check_food_collision(&mut self, food: &Food) -> bool {
        if self.head.x == food.position.x && self.head.y == food.position.y {
            self.has_eaten = true;
            true
        } else {
            false
        }
    }

    // 检测是否撞墙
    fn is_colliding_with_wall(&self, width: isize, height: isize) -> bool {
        if self.head.x < 0 || self.head.x >= width || self.head.y < 0 || self.head.y >= height {
            true
        } else {
            false
        }
    }

    // 蛇向前移动
    fn move_forward(&mut self) {
        // 移动身体
        if !self.has_eaten {
            self.body.pop();
        } else {
            self.has_eaten = false;
        }
        self.body.insert(
            0,
            Position {
                x: self.head.x,
                y: self.head.y,
            },
        );

        // 移动头部
        match self.direction {
            Direction::Up => self.head.y -= 1,
            Direction::Down => self.head.y += 1,
            Direction::Left => self.head.x -= 1,
            Direction::Right => self.head.x += 1,
        }
    }
}

// 游戏状态
enum GameState {
    // Paused,
    Started,
    Ended,
}
struct State(GameState);

// 食物
struct Food {
    position: Position,
}
impl Food {
    fn new(position: Position) -> Self {
        Self { position }
    }
    fn gen(&mut self, x: isize, y: isize) {
        self.position.x = rand::thread_rng().gen_range(0..x);
        self.position.y = rand::thread_rng().gen_range(0..y);
    }
}

// 绘制蛇身体时用到，判断点是否在蛇身体上
fn point_in_set(point: &Position, set: &Vec<Position>) -> bool {
    for p in set {
        if p.x == point.x && p.y == point.y {
            return true;
        }
    }
    false
}

fn main() {
    // 初始化
    // raw_mode
    let _ = crossterm::terminal::enable_raw_mode();
    // 初始游戏状态为开始状态
    let game_state = Arc::new(Mutex::new(State(GameState::Started)));

    // 地图
    const WIDTH: isize = 40;
    const HEIGHT: isize = 20;

    // 刷新间隔 ms
    const FLASH_TIME: u64 = 500;

    // 食物
    let food = Arc::new(Mutex::new(Food::new(Position { x: 15, y: 15 })));
    // 蛇
    let snake = Arc::new(Mutex::new(Snake::new(10, 10)));

    // 克隆，将所有权转移至闭包内部
    let snake_clone = Arc::clone(&snake);
    let food_clone = food.clone();
    let game_state_clone = game_state.clone();
    //  绘制线程
    let render_thread = std::thread::spawn(move || {
        loop {
            let snake = snake_clone.lock().unwrap();
            // println!("绘制： {},{}", snake.head.x, snake.head.y);
            // 清屏
            print!("\x1B[2J\x1B[1;1H");

            // 绘制上墙
            for _ in 0..WIDTH + 2 {
                print!("■");
            }
            print!("\n\r");

            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    // 绘制左墙
                    if x == 0 {
                        print!("■");
                    }

                    let food = food_clone.lock().unwrap();

                    if snake.head.x == x && snake.head.y == y {
                        print!("{}", "☺");
                    } else if point_in_set(&Position { x, y }, &snake.body) {
                        print!("{}", "○");
                    } else if food.position.x == x && food.position.y == y {
                        print!("{}", "●");
                    } else {
                        print!("{}", " ");
                    }

                    // 绘制右墙
                    if x == WIDTH - 1 {
                        print!("{}", "■");
                    }
                }
                print!("\n\r");
            }

            // 绘制下墙
            for _ in 0..WIDTH + 2 {
                print!("■");
            }
            print!("\n\r");

            // 释放锁
            drop(snake);
            // 刷新时间
            std::thread::sleep(std::time::Duration::from_millis(FLASH_TIME));

            if let GameState::Ended = game_state_clone.lock().unwrap().0 {
                println!("You are DEAD!");
                break;
            }
        }
    });

    // 克隆，将所有权转移至闭包内部
    let snake_clone = Arc::clone(&snake);
    let food_clone = food.clone();
    let game_state_clone = game_state.clone();
    // 向前移动线程
    let run_thread = std::thread::spawn(move || loop {
        let mut snake = snake_clone.lock().unwrap();
        let mut food = food_clone.lock().unwrap();

        // 向前移动
        snake.move_forward();

        // 吃到食物
        if snake.check_food_collision(&food) {
            food.gen(WIDTH - 1, HEIGHT - 1);
        }

        // 撞墙
        if snake.is_colliding_with_wall(WIDTH, HEIGHT) {
            game_state_clone.lock().unwrap().0 = GameState::Ended;
            break;
        }

        // 释放锁
        drop(snake);
        drop(food);

        std::thread::sleep(std::time::Duration::from_millis(FLASH_TIME));
    });

    //事件线程
    let snake_clone = Arc::clone(&snake);
    let event_thread = std::thread::spawn(move || loop {
        // 匹配上下左右及退出按键
        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press,
            ..
        }) = event::read().unwrap()
        {
            match code {
                event::KeyCode::Left => snake_clone.lock().unwrap().direction = Direction::Left,
                event::KeyCode::Right => snake_clone.lock().unwrap().direction = Direction::Right,
                event::KeyCode::Up => snake_clone.lock().unwrap().direction = Direction::Up,
                event::KeyCode::Down => snake_clone.lock().unwrap().direction = Direction::Down,
                event::KeyCode::Char('q') | event::KeyCode::Esc => {
                    // 退出游戏
                    let _ = crossterm::terminal::disable_raw_mode();
                    std::process::exit(0);
                }
                _ => (),
            }
        }
    });

    // 等待线程结束
    let _ = render_thread.join();
    let _ = run_thread.join();
    let _ = event_thread.join();
}
