use std::{cmp::Ordering, slice::Iter};

use macroquad::prelude::*;
// use std::borrow::BorrowMut;
// use std::cell::RefCell;
// use std::ops::Index;
// use std::rc::Rc;

const SQUARE_SCALE: f32 = 20.0;
const CALC_SSCALE: f32 = 14.14213562;
const CALC_SSCALE_S: f32 = 13.0;
const CALC_SSCALE_XS: f32 = 8.0;

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn all() -> Iter<'static, Direction> {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
        .iter()
    }

    pub fn delta_dirs() -> Iter<'static, (f32, f32)> {
        [
            (SQUARE_SCALE, 0.0),
            (0.0, -SQUARE_SCALE),
            (-SQUARE_SCALE, 0.0),
            (0.0, SQUARE_SCALE),
        ]
        .iter()
    }
}

struct Walls {
    walls: Vec<(f32, f32)>,
}

impl Walls {
    pub fn new() -> Walls {
        Walls { walls: Vec::new() }
    }

    pub fn try_add(&mut self, pos: (f32, f32)) {
        if !self.walls.contains(&pos) {
            self.walls.push(pos)
        }
    }

    pub fn try_remove(&mut self, pos: (f32, f32)) {
        if let Some(i) = self.walls.iter().position(|&w| w == pos) {
            self.walls.remove(i);
        }
    }
    pub fn draw(&self) {
        self.walls.iter().for_each(|(x, y)| {
            draw_poly(*x, *y, 4, CALC_SSCALE, 45.0, GRAY);
        })
    }

    pub fn len(&self) -> usize {
        self.walls.len()
    }

    pub fn exists(&self, pos: &(f32, f32)) -> bool {
        self.walls.iter().any(|w| w == pos)
    }
}

struct Arena {
    pub nodes: Vec<Node>,
    pub start_pos: (f32, f32),
    pub greatest_weight: f32,
    pub found_objective: Option<usize>,
    pub open: Vec<usize>,
}

impl Arena {
    pub fn new(start_pos: (f32, f32)) -> Self {
        Arena {
            nodes: Vec::new(),
            start_pos,
            greatest_weight: 0.0,
            found_objective: None,
            open: vec![0],
        }
    }
    pub fn new_node(&mut self, pos: (f32, f32)) -> usize {
        let next_index = self.nodes.len();

        self.nodes.push(Node {
            parent: None,
            index: next_index,
            pos,
            cost: 0.0,
            priority: 0.0,
        });

        next_index
    }

    pub fn new_child(&mut self, parent: usize, pos: (f32, f32), cost: f32, priority: f32) -> usize {
        let next_index = self.nodes.len();

        if cost as f32 > self.greatest_weight {
            self.greatest_weight = cost as f32
        }
        self.nodes.push(Node {
            parent: Some(parent),
            index: next_index,
            pos,
            cost,
            priority,
        });

        next_index
    }

    pub fn draw(&self) {
        self.nodes.iter().for_each(|n| {
            let (x, y) = n.pos;
            let r = n.cost / self.greatest_weight;
            draw_poly(
                x,
                y,
                4,
                CALC_SSCALE,
                45.0,
                Color::from_vec(vec4(0.0, 1.0 - r, 1.0 - r, 1.0)),
            );
        });
    }

    pub fn breadth_first_search(
        &mut self,
        walls: &Walls,
        objective: &(f32, f32),
        instant: &bool,
    ) -> bool {
        if *instant {
            loop {
                if self.breadth_first_search(walls, objective, &false) {
                    return true;
                }
            }
        }

        let mut just_searched = Vec::new();
        let mut found = false;
        'outer: for i in self.open.clone() {
            for (dx, dy) in Direction::delta_dirs() {
                let node = &self.nodes[i];
                let new_pos = (node.pos.0 + dx, node.pos.1 + dy);

                if self.exists(new_pos).is_none() && !walls.exists(&new_pos) {
                    let mut weight: usize = 0;
                    node.total_steps(&mut weight, self);

                    just_searched.push(self.new_child(i, new_pos, weight as f32, 0.0));

                    if &new_pos == objective {
                        self.found_objective = Some(self.nodes.len() - 1);
                        found = true;
                        break 'outer;
                    }
                }
            }
        }
        self.open = just_searched;

        self.open.is_empty() || found
    }

    pub fn best_first_search(
        &mut self,
        walls: &Walls,
        objective: &(f32, f32),
        _instant: &bool,
    ) -> bool {
        let i = self
            .open
            .clone()
            .into_iter()
            .min_by(|n1, n2| {
                self.nodes[*n1]
                    .cost
                    .partial_cmp(&self.nodes[*n2].cost)
                    .unwrap()
            })
            .unwrap();

        let p = self.open.iter().position(|u| u == &i).unwrap();
        self.open.remove(p);

        let mut just_searched: Vec<usize> = Vec::new();

        for (dx, dy) in Direction::delta_dirs() {
            let node = &self.nodes[i];
            let new_pos = (node.pos.0 + dx, node.pos.1 + dy);
            if self.exists(new_pos).is_none() && !walls.exists(&new_pos) {
                let weight = Arena::heuristic(*objective, new_pos);
                just_searched.push(self.new_child(i, new_pos, weight, 0.0));

                if &new_pos == objective {
                    self.found_objective = Some(self.nodes.len() - 1);
                    return true;
                }
            }
        }

        just_searched.into_iter().for_each(|js| self.open.push(js));

        false
    }

    pub fn a_search_star(
        &mut self,
        walls: &Walls,
        objective: &(f32, f32),
        _instant: &bool,
    ) -> bool {
        //let mut sorted = self.open.clone();
        if self.open.is_empty() {
            return true;
        }

        let i = self
            .open
            .clone()
            .into_iter()
            .min_by(|n1, n2| {
                self.nodes[*n1]
                    .priority
                    .partial_cmp(&self.nodes[*n2].priority)
                    .unwrap()
            })
            .unwrap();

        // sorted.sort_by(|n1, n2| {
        //     self.nodes[*n1]
        //         .weight
        //         .partial_cmp(&self.nodes[*n2].weight)
        //         .unwrap()
        // });

        // for i in sorted {
        let p = self.open.iter().position(|u| u == &i).unwrap();
        self.open.remove(p);

        let mut just_searched: Vec<usize> = Vec::new();
        //self.open.remove(self.open.iter().position(|u| u ))

        for (dx, dy) in Direction::delta_dirs() {
            let node = &self.nodes[i];
            let new_pos = (node.pos.0 + dx, node.pos.1 + dy);
            //let mut steps = 0;

            //node.total_steps(&mut steps, self);
            let new_cost = node.cost + SQUARE_SCALE;

            let prev_exp = self.exists(new_pos);

            let is_none = prev_exp.is_none();

            //((steps as f32 - 1.0) * SQUARE_SCALE) + Arena::heuristic(*objective, new_pos);

            //println!("{}", weight);
            if (is_none || new_cost < prev_exp.unwrap().cost) && !walls.exists(&new_pos) {
                //let heuristic =

                let weight = new_cost + Arena::heuristic(new_pos, *objective);

                if !is_none {
                    let indx = prev_exp.unwrap().index;
                    self.nodes[indx].cost = new_cost;
                    just_searched.push(indx);
                } else {
                    just_searched.push(self.new_child(i, new_pos, new_cost, weight));
                }

                //println!("{} | {}", Arena::heuristic(new_pos, *objective), new_cost);
                //println!("{:?} {:?}", steps as f32 * 20.0, heuristic);

                //println!("{:?}", weight);

                if &new_pos == objective {
                    self.found_objective = Some(self.nodes.len() - 1);
                    return true;
                }
            }
        }

        just_searched.into_iter().for_each(|js| {
            if !self.open.contains(&js) {
                self.open.push(js)
            }
        });
        // }

        false
    }

    pub fn cached_search(&mut self, objective: &(f32, f32)) {
        let mut f = None;
        for (i, node) in self.nodes.iter().enumerate() {
            if node.pos == *objective {
                f = Some(i);
                break;
            }
        }
        self.found_objective = f;
    }

    pub fn reset(&mut self) {
        self.nodes = Vec::new();
        self.new_node(self.start_pos);
        self.found_objective = None;
        self.greatest_weight = 0.0;
        self.open = vec![0]
    }

    pub fn smart_reset(&mut self, placed: (f32, f32)) -> bool {
        if self.exists(placed).is_some() {
            self.reset();
            true
        } else {
            false
        }
    }

    pub fn i_smart_reset(&mut self, placed: (f32, f32)) -> bool {
        if self.exists(placed).is_none() {
            self.reset();
            true
        } else {
            false
        }
    }

    pub fn exists(&self, pos: (f32, f32)) -> Option<&Node> {
        self.nodes.iter().find(|n| n.pos == pos)
    }

    // pub fn manhattan_distance(&self, pos: (f32, f32)) -> f32 {
    //     let (x1, y1) = self.start_pos;
    //     let (x2, y2) = pos;

    //     ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
    // }

    pub fn distance_from_start(&self, pos: (f32, f32)) -> f32 {
        let (x1, y1) = self.start_pos;
        let (x2, y2) = pos;

        (x1 - x2).abs() + (y1 - y2).abs()
        //((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
    }

    pub fn heuristic(p1: (f32, f32), p2: (f32, f32)) -> f32 {
        let (x1, y1) = p1;
        let (x2, y2) = p2; //

        (x1 - x2).abs() + (y1 - y2).abs()
        //((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
    }
}

struct Node {
    parent: Option<usize>,
    index: usize,
    pub cost: f32,
    pub priority: f32,
    pub pos: (f32, f32),
}

impl Node {
    pub fn draw(&self, arena: &Arena) {
        let (x, y) = self.pos;
        draw_poly(x, y, 4, CALC_SSCALE, 45.0, WHITE);

        if let Some(parent) = self.parent {
            arena.nodes[parent].draw(arena)
        }
    }

    pub fn total_steps(&self, total_steps: &mut usize, arena: &Arena) {
        if let Some(parent) = self.parent {
            *total_steps += 1;
            arena.nodes[parent].total_steps(total_steps, arena)
        }
    }
}

fn round_pos(x: f32, y: f32) -> (f32, f32) {
    (
        SQUARE_SCALE * (x / SQUARE_SCALE).round(),
        SQUARE_SCALE * (y / SQUARE_SCALE).round(),
    )
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Nathans Game".to_owned(),
        window_height: 1080,
        window_width: 1920,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let (sw, sh) = (screen_width(), screen_height());
    let (cx, cy) = (sw / 2.0, sh / 2.0);

    let mut walls = Walls::new();

    (0..sw as u32).for_each(|i| {
        walls.try_add(round_pos(i as f32, -10.0));
        walls.try_add(round_pos(i as f32, sh + 10.0));
    });

    (0..sh as u32).for_each(|i| {
        walls.try_add(round_pos(-10.0, i as f32));
        walls.try_add(round_pos(sw + 10.0, i as f32));
    });

    let mut arena = Arena::new(round_pos(cx, cy));

    arena.new_node(round_pos(cx, cy));

    let mut objective = round_pos(cx - 300.0, cy - 300.0);

    let mut instant = false;
    let mut visualize = true;

    //set_cursor_grab(true);
    loop {
        let (mx, my) = mouse_position();
        let (mx, my) = round_pos(mx, my);

        if is_key_pressed(KeyCode::Space) {
            instant = !instant
        } else if is_key_pressed(KeyCode::V) {
            visualize = !visualize
        }

        if is_mouse_button_down(MouseButton::Left) {
            if is_key_down(KeyCode::LeftShift) {
                walls.try_remove((mx, my));
                arena.reset();
            } else {
                walls.try_add((mx, my));
                arena.reset();
                //arena.smart_reset((mx, my));
            }
        }

        if !arena.found_objective.is_some() {
            // arena.breadth_first_search(&walls, &objective, &(instant || !&visualize));

            // arena.a_search_star(&walls, &objective, &true);
            // if instant {
            arena.breadth_first_search(&walls, &objective, &false);
            // } else {
            //     arena.a_search_star(&walls, &objective, &false);
            // }
        }

        if is_key_pressed(KeyCode::O) {
            objective = (mx, my);
            arena.cached_search(&objective);
            arena.i_smart_reset(objective);
            //arena.reset((cx, cy));
            //last_node = arena.new_node((cx, cy));
            //last_searched = vec![last_node];
        }

        clear_background(DARKGRAY);

        // if arena.nodes.len() > 0 {
        //     arena.nodes[arena.nodes.len() - 1].draw(&arena);
        // }
        if visualize {
            arena.draw();
        }

        walls.draw();

        if let Some(i) = arena.found_objective {
            arena.nodes[i].draw(&arena);
        }

        draw_poly(
            arena.start_pos.0,
            arena.start_pos.1,
            4,
            CALC_SSCALE,
            45.0,
            GREEN,
        );

        draw_poly(objective.0, objective.1, 4, CALC_SSCALE, 45.0, GOLD);
        draw_poly(mx, my, 4, CALC_SSCALE, 45.0, WHITE);

        let fps = get_fps();
        draw_text(&format!("fps: {}", fps), 2.0, 20.0, 30.0, GREEN);
        draw_text(
            &format!("nodes: {}", arena.nodes.len()),
            2.0,
            40.0,
            30.0,
            GREEN,
        );

        next_frame().await
    }
}
