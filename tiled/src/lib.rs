use nanoserde::DeJson;

use macroquad::*;

use std::collections::HashMap;

mod error;
mod tiled;

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

pub struct Tile {
    /// id in the tileset
    pub id: u32,
    /// Each tile belongs to one tileset
    pub tileset: String,
}

pub struct Layer {
    pub objects: Vec<Object>,
    pub width: u32,
    pub height: u32,
    pub data: Vec<Option<Tile>>,
}

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

pub struct Map {
    pub layers: HashMap<String, Layer>,
    pub tilesets: HashMap<String, TileSet>,
    pub camera: Vec2,

    /// Deserialized json as is
    pub raw_tiled_map: tiled::Map,
}

impl Map {
    pub fn spr(&self, tileset: &str, sprite: u32, pos: Vec2, flip_x: bool, flip_y: bool) {
        let tileset = &self.tilesets[tileset];
        let spr_rect = tileset.sprite_rect(sprite);

        draw_texture_ex(
            tileset.texture,
            pos.x() / 8. * 0.04 - self.camera.x() / 8. * 0.04,
            pos.y() / 8. * 0.04 - self.camera.y() / 8. * 0.04,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(0.04, 0.04)),
                source: Some(Rect::new(spr_rect.x, spr_rect.y, spr_rect.w, spr_rect.h)),
                ..Default::default()
            },
        );
    }

    pub fn draw_tiles(&self, layer: &str) {
        let layer = &self.layers[layer];

        for y in 0..layer.height {
            for x in 0..layer.width {
                if let Some(tile) = &layer.data[(y * layer.width + x) as usize] {
                    self.spr(
                        &tile.tileset,
                        tile.id,
                        vec2(x as f32 * 8., y as f32 * 8.),
                        false,
                        false,
                    );
                }
            }
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
                        find_tileset(*tile).map(|tileset| Tile {
                            id: *tile - tileset.firstgid,
                            tileset: tileset.name.clone(),
                        })
                    })
                    .collect::<Vec<_>>(),
            },
        );
    }

    Ok(Map {
        layers,
        tilesets,
        camera: Vec2::new(0., 0.),
        raw_tiled_map: map,
    })
}
