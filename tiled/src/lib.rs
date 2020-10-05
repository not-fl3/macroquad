use nanoserde::DeJson;

use macroquad::prelude::*;

use std::collections::HashMap;

mod error;
mod tiled;

#[derive(Debug)]
pub enum Object {
    Rect {
        world_x: f32,
        world_y: f32,
        world_w: f32,
        world_h: f32,

        tile_x: u32,
        tile_y: u32,
        tile_w: u32,
        tile_h: u32,

        name: String,
    },
}

#[derive(Debug)]
pub struct Tile {
    /// id in the tileset
    pub id: u32,
    /// Each tile belongs to one tileset
    pub tileset: String,
    /// "type" from tiled
    pub attrs: String,
}

#[derive(Debug)]
pub struct Layer {
    pub objects: Vec<Object>,
    pub width: u32,
    pub height: u32,
    pub data: Vec<Option<Tile>>,
}

#[derive(Debug)]
pub struct TileSet {
    pub texture: Texture2D,

    pub tilewidth: i32,
    pub tileheight: i32,
    pub columns: u32,
    pub spacing: i32,
    pub margin: i32,
}

impl TileSet {
    fn sprite_rect(&self, ix: u32) -> Rect {
        let sw = self.tilewidth as f32;
        let sh = self.tileheight as f32;
        let sx = (ix % self.columns) as f32 * (sw + self.spacing as f32) + self.margin as f32;
        let sy = (ix / self.columns) as f32 * (sh + self.spacing as f32) + self.margin as f32;

        Rect::new(sx + 1., sy + 1., sw - 2., sh - 2.)
    }
}

#[derive(Debug)]
pub struct Map {
    pub layers: HashMap<String, Layer>,
    pub tilesets: HashMap<String, TileSet>,

    /// Deserialized json as is
    pub raw_tiled_map: tiled::Map,
}

impl Map {
    pub fn spr(&self, tileset: &str, sprite: u32, dest: Rect) {
        let tileset = &self.tilesets[tileset];
        let spr_rect = tileset.sprite_rect(sprite);

        draw_texture_ex(
            tileset.texture,
            dest.x,
            dest.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dest.w, dest.h)),
                source: Some(Rect::new(spr_rect.x, spr_rect.y, spr_rect.w, spr_rect.h)),
                ..Default::default()
            },
        );
    }

    pub fn draw_tiles(&self, layer: &str, dest: Rect, source: Rect) {
        let layer = &self.layers[layer];

        let spr_width = dest.w / (source.w + 1.);
        let spr_height = dest.h / (source.h + 1.);

        for y in source.y as u32..source.y as u32 + source.h as u32 + 1 {
            for x in source.x as u32..source.x as u32 + source.w as u32 + 1 {
                let pos = vec2(
                    (x - source.x as u32) as f32 / (source.w + 1.) * dest.w + dest.x,
                    (y - source.y as u32) as f32 / (source.h + 1.) * dest.h + dest.y,
                );

                if let Some(tile) = &layer.data[(y * layer.width + x) as usize] {
                    self.spr(
                        &tile.tileset,
                        tile.id,
                        Rect::new(pos.x(), pos.y(), spr_width, spr_height),
                    );
                }
            }
        }
    }

    pub fn tiles(&self, layer: &str, rect: Rect) -> TilesIterator {
        TilesIterator::new(&self.layers[layer], rect)
    }

    pub fn get_tile(&self, layer: &str, x: u32, y: u32) -> &Option<Tile> {
        let layer = &self.layers[layer];

        if x >= layer.width || y >= layer.height {
            return &None;
        }

        &layer.data[(y * layer.width + x) as usize]
    }
}

pub struct TilesIterator<'a> {
    rect: Rect,
    current: (u32, u32),
    layer: &'a Layer,
}

impl<'a> TilesIterator<'a> {
    fn new(layer: &'a Layer, rect: Rect) -> Self {
        let current = (rect.x as u32, rect.y as u32);

        TilesIterator {
            layer,
            rect,
            current,
        }
    }
}

impl<'a> Iterator for TilesIterator<'a> {
    type Item = (u32, u32, &'a Option<Tile>);

    fn next(&mut self) -> Option<Self::Item> {
        let next_x;
        let next_y;

        if self.current.0 + 1 >= self.rect.x as u32 + self.rect.w as u32 {
            next_x = self.rect.x as u32;
            next_y = self.current.1 + 1;
        } else {
            next_x = self.current.0 + 1;
            next_y = self.current.1;
        }
        self.current = (next_x, next_y);

        if next_y > self.rect.y as u32 + self.rect.h as u32 {
            None
        } else {
            Some((
                next_x,
                next_y,
                &self.layer.data[(next_y * self.layer.width + next_x) as usize],
            ))
        }
    }
}

/// Load Tiled tileset from given json string
pub fn load_map(data: &str, textures: &[(&str, Texture2D)]) -> Result<Map, error::Error> {
    let map: tiled::Map = DeJson::deserialize_json(data)?;

    let mut layers = HashMap::new();
    let mut tilesets = HashMap::new();

    for tileset in &map.tilesets {
        let texture = textures
            .iter()
            .find(|(name, _)| *name == tileset.image)
            .ok_or(error::Error::TextureNotFound {
                texture: tileset.image.clone(),
            })?
            .1;

        tilesets.insert(
            tileset.name.clone(),
            TileSet {
                texture,
                columns: tileset.columns as _,
                margin: tileset.margin,
                spacing: tileset.spacing,
                tilewidth: tileset.tilewidth,
                tileheight: tileset.tileheight,
            },
        );
    }

    for layer in &map.layers {
        if layers.contains_key(&layer.name) {
            return Err(error::Error::NonUniqueLayerName {
                layer: layer.name.clone(),
            });
        }

        let tile_width = map.tilewidth as f32;
        let tile_height = map.tileheight as f32;

        let mut objects = vec![];
        for object in &layer.objects {
            objects.push(Object::Rect {
                world_x: object.x,
                world_y: object.y,
                world_w: object.width,
                world_h: object.height,

                tile_x: (object.x / tile_width) as u32,
                tile_y: (object.y / tile_height) as u32,
                tile_w: (object.width / tile_width) as u32,
                tile_h: (object.height / tile_height) as u32,
                name: object.name.clone(),
            });
        }

        let find_tileset = |tile: u32| {
            map.tilesets.iter().find(|tileset| {
                tile >= tileset.firstgid && tile < tileset.firstgid + tileset.tilecount
            })
        };

        layers.insert(
            layer.name.clone(),
            Layer {
                objects,
                width: layer.width,
                height: layer.height,
                data: layer
                    .data
                    .iter()
                    .map(|tile| {
                        find_tileset(*tile).map(|tileset| {
                            let attrs = tileset
                                .tiles
                                .iter()
                                .find(|t| t.id as u32 == *tile - tileset.firstgid)
                                .and_then(|tile| tile.ty.clone())
                                .unwrap_or("".to_owned());

                            Tile {
                                id: *tile - tileset.firstgid,
                                tileset: tileset.name.clone(),
                                attrs,
                            }
                        })
                    })
                    .collect::<Vec<_>>(),
            },
        );
    }

    Ok(Map {
        layers,
        tilesets,
        raw_tiled_map: map,
    })
}
