use nanoserde::DeJson;

use macroquad::prelude::*;

use std::collections::HashMap;

mod error;
mod tiled;

pub use tiled::layer::Property;

#[derive(Debug)]
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

impl Map {
    pub fn spr(&self, tileset: &str, sprite: u32, dest: Rect) {
        if self.tilesets.contains_key(tileset) == false {
            panic!(
                "No such tilest: {}, tilesets available: {:?}",
                tileset,
                self.tilesets.keys()
            )
        }
        let tileset = &self.tilesets[tileset];
        let spr_rect = tileset.sprite_rect(sprite);

        draw_texture_ex(
            tileset.texture,
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
                ..Default::default()
            },
        );
    }

    pub fn spr_ex(&self, tileset: &str, source: Rect, dest: Rect) {
        let tileset = &self.tilesets[tileset];

        draw_texture_ex(
            tileset.texture,
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

        for y in source.y as u32..source.y as u32 + source.h as u32 {
            for x in source.x as u32..source.x as u32 + source.w as u32 {
                let pos = vec2(
                    (x - source.x as u32) as f32 / source.w * dest.w + dest.x,
                    (y - source.y as u32) as f32 / source.h * dest.h + dest.y,
                );

                if let Some(tile) = &layer.data[(y * layer.width + x) as usize] {
                    self.spr(
                        &tile.tileset,
                        tile.id,
                        Rect::new(pos.x, pos.y, spr_width, spr_height),
                    );
                }
            }
        }
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
            map_tilesets.iter().find(|tileset| {
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
