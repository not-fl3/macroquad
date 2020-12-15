use macroquad::math::{vec2, Rect, Vec2};

use std::collections::HashSet;

pub struct StaticTiledLayer {
    static_colliders: Vec<bool>,
    tile_width: f32,
    tile_height: f32,
    width: usize,
    tag: u8,
}

pub struct World {
    static_tiled_layers: Vec<StaticTiledLayer>,
    solids: Vec<(Solid, Collider)>,
    actors: Vec<(Actor, Collider)>,
}

#[derive(Clone, Debug)]
struct Collider {
    collidable: bool,
    squished: bool,
    pos: Vec2,
    width: i32,
    height: i32,
    x_remainder: f32,
    y_remainder: f32,
    squishers: HashSet<Solid>,
}

impl Collider {
    pub fn rect(&self) -> Rect {
        Rect::new(
            self.pos.x,
            self.pos.y,
            self.width as f32,
            self.height as f32,
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Actor(usize);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Solid(usize);

impl World {
    pub fn new() -> World {
        World {
            static_tiled_layers: vec![],
            actors: vec![],
            solids: vec![],
        }
    }

    pub fn add_static_tiled_layer(
        &mut self,
        static_colliders: Vec<bool>,
        tile_width: f32,
        tile_height: f32,
        width: usize,
        tag: u8,
    ) {
        self.static_tiled_layers.push(StaticTiledLayer {
            static_colliders,
            tile_width,
            tile_height,
            width,
            tag,
        });
    }
    pub fn add_actor(&mut self, pos: Vec2, width: i32, height: i32) -> Actor {
        let actor = Actor(self.actors.len());

        self.actors.push((
            actor,
            Collider {
                collidable: true,
                squished: false,
                pos,
                width,
                height,
                x_remainder: 0.,
                y_remainder: 0.,
                squishers: HashSet::new(),
            },
        ));

        actor
    }

    pub fn add_solid(&mut self, pos: Vec2, width: i32, height: i32) -> Solid {
        let solid = Solid(self.solids.len());

        self.solids.push((
            solid,
            Collider {
                collidable: true,
                squished: false,
                pos,
                width,
                height,
                x_remainder: 0.,
                y_remainder: 0.,
                squishers: HashSet::new(),
            },
        ));

        solid
    }

    pub fn move_v(&mut self, actor: Actor, dy: f32) -> bool {
        let id = actor.0;
        let mut collider = self.actors[id].1.clone();

        collider.y_remainder += dy;

        let mut move_ = collider.y_remainder.round() as i32;
        if move_ != 0 {
            collider.y_remainder -= move_ as f32;
            let sign = move_.signum();

            while move_ != 0 {
                if self.collide_solids(
                    collider.pos + vec2(0., sign as f32),
                    collider.width,
                    collider.height,
                ) == false
                {
                    collider.pos.y += sign as f32;
                    move_ -= sign;
                } else {
                    self.actors[id].1 = collider;

                    return false;
                }
            }
        }
        self.actors[id].1 = collider;
        true
    }

    pub fn move_h(&mut self, actor: Actor, dy: f32) -> bool {
        let id = actor.0;
        let mut collider = self.actors[id].1.clone();
        collider.x_remainder += dy;

        let mut move_ = collider.x_remainder.round() as i32;
        if move_ != 0 {
            collider.x_remainder -= move_ as f32;
            let sign = move_.signum();

            while move_ != 0 {
                if self.collide_solids(
                    collider.pos + vec2(sign as f32, 0.),
                    collider.width,
                    collider.height,
                ) == false
                {
                    collider.pos.x += sign as f32;
                    move_ -= sign;
                } else {
                    self.actors[id].1 = collider;
                    return false;
                }
            }
        }
        self.actors[id].1 = collider;
        true
    }

    pub fn solid_move(&mut self, solid: Solid, dx: f32, dy: f32) {
        let collider = &mut self.solids[solid.0].1;

        collider.x_remainder += dx;
        collider.y_remainder += dy;
        let move_x = collider.x_remainder.round() as i32;
        let move_y = collider.y_remainder.round() as i32;

        let mut riding_actors = vec![];
        let mut pushing_actors = vec![];

        let riding_rect = Rect::new(
            collider.pos.x,
            collider.pos.y - 1.0,
            collider.width as f32,
            1.0,
        );
        let pushing_rect = Rect::new(
            collider.pos.x + move_x as f32,
            collider.pos.y,
            collider.width as f32 - 1.0,
            collider.height as f32,
        );

        for (actor, actor_collider) in &mut self.actors {
            let rider_rect = Rect::new(
                actor_collider.pos.x,
                actor_collider.pos.y + actor_collider.height as f32 - 1.0,
                actor_collider.width as f32,
                1.0,
            );

            if riding_rect.overlaps(&rider_rect) {
                riding_actors.push(*actor);
            } else if pushing_rect.overlaps(&actor_collider.rect())
                && actor_collider.squished == false
            {
                pushing_actors.push(*actor);
            }

            if pushing_rect.overlaps(&actor_collider.rect()) == false {
                actor_collider.squishers.remove(&solid);
                if actor_collider.squishers.len() == 0 {
                    actor_collider.squished = false;
                }
            }
        }

        self.solids[solid.0].1.collidable = false;
        for actor in riding_actors {
            self.move_h(actor, move_x as f32);
        }
        for actor in pushing_actors {
            let squished = !self.move_h(actor, move_x as f32);
            if squished {
                self.actors[actor.0].1.squished = true;
                self.actors[actor.0].1.squishers.insert(solid);
            }
        }
        self.solids[solid.0].1.collidable = true;

        let collider = &mut self.solids[solid.0].1;
        if move_x != 0 {
            collider.x_remainder -= move_x as f32;
            collider.pos.x += move_x as f32;
        }
        if move_y != 0 {
            collider.y_remainder -= move_y as f32;
            collider.pos.y += move_y as f32;
        }
    }

    pub fn solid_at(&self, pos: Vec2) -> bool {
        self.tag_at(pos, 1)
    }

    pub fn tag_at(&self, pos: Vec2, tag: u8) -> bool {
        for StaticTiledLayer {
            tile_width,
            tile_height,
            width,
            static_colliders,
            tag: layer_tag,
        } in &self.static_tiled_layers
        {
            let y = (pos.y / tile_width) as i32;
            let x = (pos.x / tile_height) as i32;
            let ix = y * (*width as i32) + x;

            if ix >= 0 && ix < static_colliders.len() as i32 && static_colliders[ix as usize] {
                return *layer_tag == tag;
            }
        }

        self.solids
            .iter()
            .any(|solid| solid.1.collidable && solid.1.rect().contains(pos))
    }

    pub fn collide_solids(&self, pos: Vec2, width: i32, height: i32) -> bool {
        self.solid_at(pos)
            || self.solid_at(pos + vec2(width as f32 - 1., 0.0))
            || self.solid_at(pos + vec2(0.0, height as f32 - 1.))
            || self.solid_at(pos + vec2(width as f32 - 1., height as f32 - 1.))
    }

    pub fn collide_tag(&self, tag: u8, pos: Vec2, width: i32, height: i32) -> bool {
        self.tag_at(pos, tag)
            || self.tag_at(pos + vec2(width as f32 - 1., 0.0), tag)
            || self.tag_at(pos + vec2(0.0, height as f32 - 1.), tag)
            || self.tag_at(pos + vec2(width as f32 - 1., height as f32 - 1.), tag)
    }

    pub fn squished(&self, actor: Actor) -> bool {
        self.actors[actor.0].1.squished
    }

    pub fn actor_pos(&self, actor: Actor) -> Vec2 {
        self.actors[actor.0].1.pos
    }

    pub fn solid_pos(&self, solid: Solid) -> Vec2 {
        self.solids[solid.0].1.pos
    }

    pub fn collide_check(&self, collider: Actor, pos: Vec2) -> bool {
        let collider = &self.actors[collider.0];

        self.collide_solids(pos, collider.1.width, collider.1.height)
    }
}
