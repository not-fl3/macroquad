use nanoserde::DeJson;

use std::collections::HashMap;

/// https://doc.mapeditor.org/en/stable/reference/json-map-format/#json-chunk
#[derive(Clone, Debug, Default, DeJson)]
#[nserde(default)]
pub struct Chunk {
    /// Array of unsigned int (GIDs) or base64-encoded data
    pub data: Vec<u32>,
    /// Height in tiles
    pub height: usize,
    /// Width in tiles
    pub width: usize,
    /// X coordinate in tiles
    pub x: i32,
    /// Y coordinate in tiles
    pub y: i32,
}

#[derive(Clone, Debug, Default, DeJson)]
#[nserde(default)]
pub struct Layer {
    /// Array of chunks (optional). tilelayer only.
    pub chunks: Option<Vec<Chunk>>,
    pub name: String,
    pub opacity: f32,
    pub properties: Option<HashMap<String, String>>,
    pub visible: bool,
    pub width: u32,
    pub height: u32,
    #[nserde(rename = "type")]
    pub ty: String,

    /// for type = "tilelayer"
    pub data: Vec<u32>,

    /// for type = "objectlayer"
    pub draworder: Option<String>,
    #[nserde(default)]
    pub objects: Vec<Object>,
    /// Horizontal layer offset in pixels (default: 0)
    pub offsetx: Option<i32>,
    /// Vertical layer offset in pixels (default: 0)
    pub offsety: Option<i32>,
    /// Horizontal layer offset in tiles. Always 0.
    pub x: Option<f32>,
    /// Vertical layer offset in tiles. Always 0.
    pub y: Option<f32>,
}

#[derive(Clone, Debug, Default, DeJson)]
#[nserde(default)]
pub struct Object {
    pub id: u32,
    pub name: String,

    #[nserde(rename = "type")]
    pub ty: String,
    pub gid: Option<u32>,
    pub ellipse: Option<bool>,
    pub polygon: Option<Vec<PolyPoint>>,

    pub properties: HashMap<String, String>,
    pub rotation: f32,
    pub visible: bool,

    pub height: f32,
    pub width: f32,

    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, Debug, DeJson)]
pub struct PolyPoint {
    pub x: f32,
    pub y: f32,
}
