use crate::basic::{Button, Component, Panel, Windows};
use crate::UPIntrFreeCell;
use alloc::collections::{BTreeSet, VecDeque};
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::{format, vec};
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::{PixelIteratorExt, Point, RgbColor};
use lazy_static::lazy_static;
use log::{error, info};
use virtio_input_decoder::Key;

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub enum Status {
    Wait,
    Alive,
    Dead,
}

// 对于吃东西事件和键盘事件
// 两者的更新策略不同
#[derive(Debug)]
enum Event {
    Eat,
    Keyboard(Direction),
}

pub struct Snake {
    score: u32,
    food: Option<Point>,
    status: Status,
    point_set: BTreeSet<Point>,
    snake: VecDeque<Point>,
    snake_direction: VecDeque<Direction>,
    window: Arc<Windows>, //游戏窗口
    score_button: Arc<Button>,
    score_back: Arc<Panel>,
}

const SNAKE_SIZE: i32 = 10;
const SNAKE_HEAD_COLOR: Rgb888 = Rgb888::RED;
const SNAKE_COLOR: Rgb888 = Rgb888::WHITE;
const FOOD_COLOR: Rgb888 = Rgb888::RED;
const SCENE_COLOR: Rgb888 = Rgb888::BLACK;
const SCENE_SIZE: Size = Size::new(700, 600);
const SCENE_POINT: Point = Point::new(200, 50);
const SNAKE_DOT_SIZE: Size = Size::new(SNAKE_SIZE as u32, SNAKE_SIZE as u32);

lazy_static! {
    pub static ref SNAKE: UPIntrFreeCell<Option<Arc<Windows>>> =
        unsafe { UPIntrFreeCell::new(None) };
}

impl Snake {
    fn any_point_to_center_point(point: Point) -> Point {
        //根据传入的点计算其所在的节点
        //返回节点的中心坐标
        let x = point.x / SNAKE_SIZE;
        let y = point.y / SNAKE_SIZE;
        Point::new(
            x * SNAKE_SIZE + SNAKE_SIZE / 2,
            y * SNAKE_SIZE + SNAKE_SIZE / 2,
        )
    }
    #[inline]
    fn top_left_from_center(point: Point) -> Point {
        point - Point::new(SNAKE_SIZE / 2, SNAKE_SIZE / 2)
    }
    pub fn new() -> Self {
        let point = SCENE_POINT;
        let size = SCENE_SIZE;
        let mut window = Windows::new(size + Size::new(0, 25), point);
        window.set_title("Snake").set_back_ground_color(SCENE_COLOR);
        //
        let mut snake = [Point::new(200 + 350, 50 + 300); 2];
        let mut point_set = BTreeSet::new();
        point_set.insert(snake[0]);
        *SNAKE.exclusive_access() = Some(window.clone());
        let panel = Panel::new(Size::new(700, 25), Point::new(200, 50 + 600));

        let button = Button::new(
            Size::new(100, 20),
            Point::new(200 + 300, 50 + 600),
            None,
            "Score:0".to_string(),
        );
        Self {
            score: 0,
            snake: VecDeque::from(snake.to_vec()),
            food: None,
            status: Status::Wait,
            point_set,
            window,
            snake_direction: VecDeque::from(vec![Direction::Right; 2]),
            score_button: Arc::new(button),
            score_back: Arc::new(panel),
        }
    }
    fn draw_game_scene(&mut self) {
        // 绘制基础的游戏界面
        self.window.paint();
        self.score_back.paint();
        self.score_button.paint();
        let panel_board = Panel::new(Size::new(10, 600), Point::new(200, 50));
        panel_board.set_background_color(Rgb888::BLUE).paint();
        let panel_board = Panel::new(Size::new(700, 10), Point::new(200, 50));
        panel_board.set_background_color(Rgb888::BLUE).paint();
        let panel_board = Panel::new(Size::new(700, 10), Point::new(200, 650 - 10));
        panel_board.set_background_color(Rgb888::BLUE).paint();
        let panel_board = Panel::new(Size::new(10, 600), Point::new(900 - 10, 50));
        panel_board.set_background_color(Rgb888::BLUE).paint();
        //绘制蛇头
        let center = Snake::any_point_to_center_point(self.snake[0]);
        self.point_set.insert(center); //插入节点集合中
        self.snake[1] = center;
        self.snake[0] = center - Point::new(SNAKE_SIZE, 0);
        let panel_head = Panel::new(Size::new(10, 10), Snake::top_left_from_center(center));
        panel_head.set_background_color(Rgb888::RED).paint();

        let panel_tail = Panel::new(
            Size::new(10, 10),
            Snake::top_left_from_center(self.snake[0]),
        );
        panel_tail.set_background_color(SNAKE_COLOR).paint();
    }
    pub fn run(&mut self) -> Status {
        self.draw_game_scene();
        let wait_time = (0.25 * 1e9 as f64) as u64;
        let mut stamp = get_seed_from_rtc(); //当前时间戳
        loop {
            if self.food.is_none() {
                self.generate_food();
            }
            if let Some(key) = self.window.get_event() {
                match key {
                    Key::Space => {
                        info!("Space");
                        if self.status == Status::Wait {
                            self.status = Status::Alive;
                        } else {
                            self.status = Status::Wait;
                        }
                        info!("status:{:?}", self.status);
                    }
                    Key::W => {
                        if self.status != Status::Wait {
                            self.key_event(Event::Keyboard(Direction::Up));
                        }
                    }
                    Key::S => {
                        if self.status != Status::Wait {
                            self.key_event(Event::Keyboard(Direction::Down));
                        }
                    }
                    Key::A => {
                        if self.status != Status::Wait {
                            self.key_event(Event::Keyboard(Direction::Left));
                        }
                    }
                    Key::D => {
                        if self.status != Status::Wait {
                            self.key_event(Event::Keyboard(Direction::Right));
                        }
                    }
                    _ => {}
                }
            }
            self.eat_event();
            if self.status == Status::Wait {
                continue;
            }
            let current_stamp = get_seed_from_rtc();
            if current_stamp - stamp >= wait_time {
                self.key_event(Event::Keyboard(*self.snake_direction.back().unwrap()));
                stamp = current_stamp;
            }
            if self.status == Status::Dead {
                break;
            }
        }
        Status::Dead
    }
    fn generate_food(&mut self) {
        //生成食物
        let x_seed = get_seed_from_rtc();
        let mut x = oorandom::Rand32::new(x_seed).rand_u32() % 700;
        let y_seed = get_seed_from_rtc();
        let mut y = oorandom::Rand32::new(y_seed).rand_u32() % 600;
        loop {
            if x >= 10 && x <= 690 && y >= 10 && y <= 590 {
                let point = Point::new(200 + x as i32, 50 + y as i32);
                let point = Snake::any_point_to_center_point(point);
                if self.point_set.contains(&point) {
                    let x_seed = get_seed_from_rtc();
                    x = oorandom::Rand32::new(x_seed).rand_u32() % 700;
                    let y_seed = get_seed_from_rtc();
                    y = oorandom::Rand32::new(y_seed).rand_u32() % 600;
                } else {
                    self.food = Some(point);
                    break;
                }
            } else {
                let x_seed = get_seed_from_rtc();
                x = oorandom::Rand32::new(x_seed).rand_u32() % 700;
                let y_seed = get_seed_from_rtc();
                y = oorandom::Rand32::new(y_seed).rand_u32() % 600;
            }
        }
        let food_point = self.food.unwrap();
        let panel_food = Panel::new(Size::new(10, 10), Snake::top_left_from_center(food_point));
        panel_food.set_background_color(Rgb888::RED).paint();
    }

    fn eat_event(&mut self) {
        //检查当前头节点位置是否是食物
        let head_point = self.snake.back().unwrap();
        if let Some(food_point) = &self.food {
            if *head_point == *food_point {
                self.score += 1;
                self.food = None;
                let old_tail_point = self.snake.front().unwrap();
                let old_tail_direction = self.snake_direction.front().unwrap();
                let new_tail_point = match old_tail_direction {
                    Direction::Up => *old_tail_point + Point::new(0, SNAKE_SIZE),
                    Direction::Down => *old_tail_point - Point::new(0, SNAKE_SIZE),
                    Direction::Left => *old_tail_point + Point::new(SNAKE_SIZE, 0),
                    Direction::Right => *old_tail_point - Point::new(SNAKE_SIZE, 0),
                };
                let x = new_tail_point.x;
                let y = new_tail_point.y;
                if x >= 210 && x <= 900 - 10 && y >= 60 && y <= 650 - 10 {
                    self.snake.push_front(new_tail_point);
                    self.snake_direction.push_front(*old_tail_direction);
                    self.point_set.insert(new_tail_point);
                    let panel_tail = Panel::new(
                        Size::new(10, 10),
                        Snake::top_left_from_center(new_tail_point),
                    );
                    panel_tail.set_background_color(SNAKE_COLOR).paint();
                }
                self.score_button
                    .reset_text(&format!("Score:{}", self.score));
                self.score_button.cover_part(Rgb888::WHITE).paint();
            }
        }
    }
    fn key_event(&mut self, event: Event) {
        if let Event::Keyboard(direction) = event {
            let head_point = *self.snake.back().unwrap();
            let head_direction = *self.snake_direction.back().unwrap();
            let mut next_point: Option<Point> = None;
            match direction {
                Direction::Up => {
                    if head_direction != Direction::Down {
                        next_point = Some(head_point - Point::new(0, SNAKE_SIZE));
                    }
                }
                Direction::Down => {
                    if head_direction != Direction::Up {
                        next_point = Some(head_point + Point::new(0, SNAKE_SIZE));
                    }
                }
                Direction::Left => {
                    if head_direction != Direction::Right {
                        next_point = Some(head_point - Point::new(SNAKE_SIZE, 0));
                    }
                }
                Direction::Right => {
                    if head_direction != Direction::Left {
                        next_point = Some(head_point + Point::new(SNAKE_SIZE, 0));
                    }
                }
            }
            if let Some(new_point) = next_point {
                let x = new_point.x;
                let y = new_point.y;
                if self.point_set.contains(&new_point) {
                    //碰到自身
                    self.status = Status::Dead;
                } else if x <= 210 || x >= 890 || y <= 60 || y >= 640 {
                    self.status = Status::Dead;
                } else {
                    let point = head_point;
                    let old_panel_head =
                        Panel::new(SNAKE_DOT_SIZE, Snake::top_left_from_center(point));
                    old_panel_head.set_background_color(SNAKE_COLOR).paint(); //旧的头节点方向不变，颜色变为蛇身
                    let panel_head =
                        Panel::new(SNAKE_DOT_SIZE, Snake::top_left_from_center(new_point));
                    panel_head.set_background_color(Rgb888::RED).paint(); //绘制新蛇头
                    self.snake.push_back(new_point);
                    self.snake_direction.push_back(direction);
                    self.point_set.insert(new_point);

                    let tail_pont = self.snake.front().unwrap(); //蛇尾
                    let panel_tail =
                        Panel::new(SNAKE_DOT_SIZE, Snake::top_left_from_center(*tail_pont));
                    panel_tail.set_background_color(SCENE_COLOR).paint(); //蛇尾变为背景色
                    self.point_set.remove(&tail_pont);
                    self.snake.pop_front();
                    self.snake_direction.pop_front();
                }
            }
        }
    }
    fn update(&mut self, event: Event) {}
}

static SEED: AtomicU64 = AtomicU64::new(0);
/// generate seed from qemu rtc
pub fn get_seed_from_rtc() -> u64 {
    SEED.fetch_add(1, Ordering::SeqCst)
}
