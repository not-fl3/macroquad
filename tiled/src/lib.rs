use nanoserde::DeJson;

use macroquad::prelude::*;

use std::collections::HashMap;

mod error;
mod tiled;

use core::f32::consts::PI;
pub use error::Error;
pub use tiled::{Property, PropertyVal};

#[derive(Debug, Clone)]
pub struct Object {
    /// If not null - the object is (probably) a tile
    pub gid: Option<u32>,

    pub world_x: f32,
    pub world_y: f32,
    pub world_w: f32,
    pub world_h: f32,

    pub tile_x: u32,
    pub tile_y: u32,
    pub tile_w: u32,
    pub tile_h: u32,

    pub name: String,

    pub properties: HashMap<String, String>,
}

/// Flip operation application order:
/// 1. flip diagonally
/// 2. flip horizontally
/// 3. flip vertically
#[derive(Debug)]
pub struct Tile {
    /// id in the tileset
    pub id: u32,
    /// Each tile belongs to one tileset
    pub tileset: String,
    /// "type" from tiled
    pub attrs: String,
    /// Whether the tile is horizontally flipped
    pub flip_x: bool,
    /// Whether the tile is vertically flipped
    pub flip_y: bool,
    /// Whether the tile is anti-diagonally flipped
    /// (equivalent to a 90 degree clockwise rotation followed by a horizontal flip)
    pub flip_d: bool,
}

#[derive(Debug, Default)]
pub struct Layer {
    pub objects: Vec<Object>,
    pub width: u32,
    pub height: u32,
    pub data: Vec<Option<Tile>>,
    /// imagelayer
    pub opacity: f32,
    pub image: Option<Texture2D>,
    pub offsetx: Option<f32>,
    pub offsety: Option<f32>,
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

        // TODO: configure tiles margin
        Rect::new(sx + 1.1, sy + 1.1, sw - 2.2, sh - 2.2)
    }
}

#[derive(Debug)]
pub struct Map {
    pub layers: HashMap<String, Layer>,
    pub tilesets: HashMap<String, TileSet>,

    /// Deserialized json as is
    pub raw_tiled_map: tiled::Map,
}

pub struct TileFlippedParams {
    flip_x: bool,
    flip_y: bool,
    flip_d: bool,
}

impl Default for TileFlippedParams {
    fn default() -> Self {
        TileFlippedParams {
            flip_x: false,
            flip_y: false,
            flip_d: false,
        }
    }
}

impl Map {
    pub fn spr(&self, tileset: &str, sprite: u32, dest: Rect) {
        self.spr_flip(tileset, sprite, dest, TileFlippedParams::default())
    }

    pub fn spr_flip(&self, tileset: &str, sprite: u32, dest: Rect, flip: TileFlippedParams) {
        if self.tilesets.contains_key(tileset) == false {
            panic!(
                "No such tileset: {}, tilesets available: {:?}",
                tileset,
                self.tilesets.keys()
            )
        }
        let tileset = &self.tilesets[tileset];
        let spr_rect = tileset.sprite_rect(sprite);

        let rotation = if flip.flip_d {
            // diagonal flip
            -PI / 2.0
        } else {
            0.0
        };

        draw_texture_ex(
            &tileset.texture,
            dest.x,
            dest.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dest.w, dest.h)),
                source: Some(Rect::new(
                    spr_rect.x - 1.0,
                    spr_rect.y - 1.0,
                    spr_rect.w + 2.0,
                    spr_rect.h + 2.0,
                )),
                flip_x: flip.flip_x,
                flip_y: flip.flip_y ^ flip.flip_d,
                rotation: rotation,
                ..Default::default()
            },
        );
    }

    pub fn spr_ex(&self, tileset: &str, source: Rect, dest: Rect) {
        let tileset = &self.tilesets[tileset];

        draw_texture_ex(
            &tileset.texture,
            dest.x,
            dest.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(dest.w, dest.h)),
                source: Some(source),
                ..Default::default()
            },
        );
    }

    pub fn contains_layer(&self, layer: &str) -> bool {
        self.layers.contains_key(layer)
    }

    pub fn draw_tiles(&self, layer: &str, dest: Rect, source: impl Into<Option<Rect>>) {
        assert!(self.layers.contains_key(layer), "No such layer: {}", layer);

        let source = source.into().unwrap_or(Rect::new(
            0.,
            0.,
            self.raw_tiled_map.width as f32,
            self.raw_tiled_map.height as f32,
        ));
        let layer = &self.layers[layer];

        let spr_width = dest.w / source.w;
        let spr_height = dest.h / source.h;

        let mut separated_by_ts: HashMap<&str, Vec<(&Tile, Rect)>> = HashMap::new();

        for y in source.y as u32..source.y as u32 + source.h as u32 {
            for x in source.x as u32..source.x as u32 + source.w as u32 {
                if let Some(tile) = &layer
                    .data
                    .get((y * layer.width + x) as usize)
                    .unwrap_or(&None)
                {
                    if !separated_by_ts.contains_key(tile.tileset.as_str()) {
                        separated_by_ts.insert(&tile.tileset, vec![]);
                    }

                    let pos = vec2(
                        (x - source.x as u32) as f32 / source.w * dest.w + dest.x,
                        (y - source.y as u32) as f32 / source.h * dest.h + dest.y,
                    );
                    separated_by_ts
                        .get_mut(tile.tileset.as_str())
                        .unwrap()
                        .push((&tile, Rect::new(pos.x, pos.y, spr_width, spr_height)));
                }
            }
        }

        for (tileset, tileset_layer) in &separated_by_ts {
            for (tile, rect) in tileset_layer {
                self.spr_flip(
                    tileset,
                    tile.id,
                    *rect,
                    TileFlippedParams {
                        flip_x: tile.flip_x,
                        flip_y: tile.flip_y,
                        flip_d: tile.flip_d,
                    },
                );
            }
        }
    }

    pub fn draw_imglayer(&self, layer: &str, dest: Rect, source: Option<Rect>) {
        assert!(self.layers.contains_key(layer), "No such layer: {}", layer);
        let layer = &self.layers[layer];
        assert!(layer.image.is_some(), "No texture found.");
        let img_texture = layer.image.clone().unwrap();
        let dest_width_frac =
            img_texture.width() / (self.raw_tiled_map.width * self.raw_tiled_map.tilewidth) as f32;
        let dest_height_frac =
            img_texture.height() / (self.raw_tiled_map.height * self.raw_tiled_map.height) as f32;

        let source = source.unwrap_or(Rect::new(0., 0., img_texture.width(), img_texture.height()));
        draw_texture_ex(
            &img_texture,
            (layer.offsetx.unwrap() - source.x) / source.w * dest.w + dest.x,
            (layer.offsety.unwrap() - source.y) / source.h * dest.h + dest.y,
            Color {
                r: 255.,
                g: 255.,
                b: 255.,
                a: layer.opacity,
            },
            DrawTextureParams {
                dest_size: Some(vec2(dest.w * dest_width_frac, dest.h * dest_height_frac)),
                source: Some(source),
                ..Default::default()
            },
        );
    }

    pub fn tiles(&self, layer: &str, rect: impl Into<Option<Rect>>) -> TilesIterator {
        assert!(self.layers.contains_key(layer), "No such layer: {}", layer);

        let rect = rect.into().unwrap_or(Rect::new(
            0.,
            0.,
            self.raw_tiled_map.width as f32,
            self.raw_tiled_map.height as f32,
        ));
        TilesIterator::new(&self.layers[layer], rect)
    }

    pub fn get_tile(&self, layer: &str, x: u32, y: u32) -> &Option<Tile> {
        assert!(self.layers.contains_key(layer), "No such layer: {}", layer);

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

        if next_y >= self.rect.y as u32 + self.rect.h as u32 {
            return None;
        }

        let res = Some((
            self.current.0,
            self.current.1,
            &self.layer.data[(self.current.1 * self.layer.width + self.current.0) as usize],
        ));
        self.current = (next_x, next_y);
        res
    }
}

/// Load Tiled tile map from given json string.
/// "data" is a tiled json content.
/// "textures" is a map from the name used in the json to macroquad texture.
/// "external_tilesets" is a map of tileset name to tileset json content.
/// "external_tilesets" is used when in tiled the "source" field is used instead of embedded tileset.
pub fn load_map(
    data: &str,
    textures: &[(&str, Texture2D)],
    external_tilesets: &[(&str, &str)],
) -> Result<Map, error::Error> {
    // Tiled reserves 4 high bits for flip flags
    const TILE_FLIP_FLAGS: u32 = 0b11110000000000000000000000000000;

    let map: tiled::Map = DeJson::deserialize_json(data)?;

    let mut layers = HashMap::new();
    let mut tilesets = HashMap::new();
    let mut map_tilesets = vec![];

    for tileset in &map.tilesets {
        let tileset = if tileset.source.is_empty() {
            tileset.clone()
        } else {
            let tileset_data = external_tilesets
                .iter()
                .find(|(name, _)| *name == &tileset.source)
                .unwrap();
            let mut map_tileset: tiled::Tileset = DeJson::deserialize_json(&tileset_data.1)?;
            map_tileset.firstgid = tileset.firstgid;
            map_tileset
        };

        let texture = &textures
            .iter()
            .find(|(name, _)| *name == tileset.image)
            .ok_or(error::Error::TextureNotFound {
                texture: tileset.image.clone(),
            })?
            .1;

        tilesets.insert(
            tileset.name.clone(),
            TileSet {
                texture: texture.clone(),
                columns: tileset.columns as _,
                margin: tileset.margin,
                spacing: tileset.spacing,
                tilewidth: tileset.tilewidth,
                tileheight: tileset.tileheight,
            },
        );

        map_tilesets.push(tileset);
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
            objects.push(Object {
                gid: object.gid,
                world_x: object.x,
                world_y: object.y,
                world_w: object.width,
                world_h: object.height,

                tile_x: (object.x / tile_width) as u32,
                tile_y: (object.y / tile_height) as u32,
                tile_w: (object.width / tile_width) as u32,
                tile_h: (object.height / tile_height) as u32,
                name: object.name.clone(),
                properties: object
                    .properties
                    .iter()
                    .map(|property| (property.name.to_string(), property.value.to_string()))
                    .collect(),
            });
        }

        let find_tileset = |tile: u32| {
            // Discard flip flags
            let tile = tile & !TILE_FLIP_FLAGS;
            map_tilesets.iter().find(|tileset| {
                tile >= tileset.firstgid && tile < tileset.firstgid + tileset.tilecount
            })
        };

        layers.insert(
            layer.name.clone(),
            match layer.ty.as_str() {
                "tilelayer" | "objectgroup" => Layer {
                    objects,
                    width: layer.width,
                    height: layer.height,
                    data: layer
                        .data
                        .iter()
                        .map(|tile| {
                            find_tileset(*tile).map(|tileset| {
                                let flip_flags = (*tile & TILE_FLIP_FLAGS) >> 28;
                                let tile = *tile & !TILE_FLIP_FLAGS;

                                let attrs = tileset
                                    .tiles
                                    .iter()
                                    .find(|t| t.id as u32 == tile - tileset.firstgid)
                                    .and_then(|tile| tile.ty.clone())
                                    .unwrap_or("".to_owned());

                                Tile {
                                    id: tile - tileset.firstgid,
                                    tileset: tileset.name.clone(),
                                    attrs,
                                    flip_x: (flip_flags & 0b1000) != 0,
                                    flip_y: (flip_flags & 0b0100) != 0,
                                    flip_d: (flip_flags & 0b0010) != 0,
                                }
                            })
                        })
                        .collect::<Vec<_>>(),
                    opacity: layer.opacity,
                    ..Default::default()
                },
                "imagelayer" => {
                    let img_name = layer.image.clone().unwrap();
                    if img_name == "" {
                        continue;
                    }
                    let offsetx = match layer.offsetx {
                        Some(x) => Some(x as f32),
                        None => Some(0f32),
                    };
                    let offsety = match layer.offsety {
                        Some(y) => Some(y as f32),
                        None => Some(0f32),
                    };
                    let img_texture = &textures
                        .iter()
                        .find(|(name, _)| *name == img_name)
                        .ok_or(error::Error::TextureNotFound { texture: img_name })?
                        .1;

                    Layer {
                        image: Some(img_texture.clone()),
                        opacity: layer.opacity,
                        offsetx,
                        offsety,
                        ..Default::default()
                    }
                }
                layer_type => {
                    return Err(error::Error::LayerTypeNotFound {
                        layer_type: layer_type.to_string(),
                    })
                }
            },
        );
    }

    // Some external tilesets could be resolved, so we
    // include the new "map_tilesets"
    Ok(Map {
        layers,
        tilesets,
        raw_tiled_map: tiled::Map {
            tilesets: map_tilesets,
            ..map
        },
    })
}
