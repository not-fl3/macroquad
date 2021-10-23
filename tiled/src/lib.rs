use nanoserde::DeJson;

use macroquad::prelude::*;

use std::collections::{BTreeMap, HashMap};

mod error;
mod tiled;

pub use error::Error;
pub use tiled::layer::Property;

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
    pub data: LayerData,
}

impl Layer {
    #[inline]
    fn get_tile(&self, x: i32, y: i32) -> &Option<Tile> {
        match &self.data {
            LayerData::Plain(tiles) => &tiles[((y as u32) * self.width + (x as u32)) as usize],
            LayerData::Chunks(chunks) => {
                let chunk = chunks.iter().find(|chunk| {
                    x >= chunk.x
                        && x < chunk.x + chunk.width as i32
                        && y >= chunk.y
                        && y < chunk.y + chunk.height as i32
                });

                if let Some(chunk) = chunk {
                    let chunk_x = (x - chunk.x) as usize;
                    let chunk_y = (y - chunk.y) as usize;

                    &chunk.data[(chunk_y * chunk.width + chunk_x) as usize]
                } else {
                    &None
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum LayerData {
    Plain(Vec<Option<Tile>>),
    Chunks(Vec<Chunk>),
}

#[derive(Debug)]
pub struct Chunk {
    pub data: Vec<Option<Tile>>,
    pub height: usize,
    pub width: usize,
    pub x: i32,
    pub y: i32,
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

    pub start_x: i32,
    pub start_y: i32,

    /// Deserialized json as is
    pub raw_tiled_map: tiled::Map,
}

impl Map {
    pub fn spr(&self, tileset: &str, sprite: u32, dest: Rect) {
        if self.tilesets.contains_key(tileset) == false {
            panic!(
                "No such tileset: {}, tilesets available: {:?}",
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
            self.start_x as f32,
            self.start_y as f32,
            self.raw_tiled_map.width as f32,
            self.raw_tiled_map.height as f32,
        ));
        let layer = &self.layers[layer];

        let spr_width = dest.w / source.w;
        let spr_height = dest.h / source.h;

        for y in source.y as i32..source.y as i32 + source.h as i32 {
            for x in source.x as i32..source.x as i32 + source.w as i32 {
                let pos = vec2(
                    (x - source.x as i32) as f32 / source.w * dest.w + dest.x,
                    (y - source.y as i32) as f32 / source.h * dest.h + dest.y,
                );

                if let Some(tile) = &layer.get_tile(x, y) {
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

        layer.get_tile(x as i32, y as i32)
    }
}

pub struct TilesIterator<'a> {
    rect: Rect,
    current: (i32, i32),
    layer: &'a Layer,
}

impl<'a> TilesIterator<'a> {
    fn new(layer: &'a Layer, rect: Rect) -> Self {
        let current = (rect.x as i32, rect.y as i32);

        TilesIterator {
            layer,
            rect,
            current,
        }
    }
}

impl<'a> Iterator for TilesIterator<'a> {
    type Item = (i32, i32, &'a Option<Tile>);

    fn next(&mut self) -> Option<Self::Item> {
        let next_x;
        let next_y;

        if self.current.0 + 1 >= self.rect.x as i32 + self.rect.w as i32 {
            next_x = self.rect.x as i32;
            next_y = self.current.1 + 1;
        } else {
            next_x = self.current.0 + 1;
            next_y = self.current.1;
        }

        if next_y >= self.rect.y as i32 + self.rect.h as i32 {
            return None;
        }

        let res = Some((
            self.current.0,
            self.current.1,
            self.layer.get_tile(self.current.0, self.current.1),
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
    let map: tiled::Map =
        DeJson::deserialize_json(data).map_err(|err| error::Error::TileMapLoadingFailed {
            inner_error: Box::new(err.into()),
        })?;

    let mut layers = HashMap::new();
    let mut tilesets = HashMap::new();
    let mut map_tilesets = vec![];

    let mut start_x = 0;
    let mut start_y = 0;

    for tileset in &map.tilesets {
        let tileset = if tileset.source.is_empty() {
            tileset.clone()
        } else {
            let tileset_data = external_tilesets
                .iter()
                .find(|(name, _)| *name == &tileset.source)
                .expect(&format!(
                    "Expected to find {} in external tilesets",
                    tileset.source
                ));
            let mut map_tileset: tiled::Tileset = DeJson::deserialize_json(&tileset_data.1)
                .map_err(|err| error::Error::TilesetLoadingFailed {
                    tileset: tileset.source.clone(),
                    inner_error: Box::new(err.into()),
                })?;
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

        start_x = layer.startx.map(|sx| start_x.min(sx)).unwrap_or(start_x);
        start_y = layer.starty.map(|sy| start_y.min(sy)).unwrap_or(start_y);

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

        let map_tiles = |tile: &u32| {
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
        };

        let layer_data = if let Some(chunks) = &layer.chunks {
            LayerData::Chunks(
                chunks
                    .iter()
                    .map(|chunk| Chunk {
                        data: chunk.data.iter().map(map_tiles).collect(),
                        width: chunk.height,
                        height: chunk.height,
                        x: chunk.x,
                        y: chunk.y,
                    })
                    .collect(),
            )
        } else {
            LayerData::Plain(layer.data.iter().map(map_tiles).collect::<Vec<_>>())
        };

        layers.insert(
            layer.name.clone(),
            Layer {
                objects,
                width: layer.width,
                height: layer.height,
                data: layer_data,
            },
        );
    }

    Ok(Map {
        layers,
        tilesets,
        start_x,
        start_y,
        raw_tiled_map: map,
    })
}
