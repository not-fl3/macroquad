#[derive(Default)]
pub struct RGBA8Buffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

pub fn decode(bytes: &[u8]) -> Result<RGBA8Buffer, Box<std::error::Error>> {
    if bytes.len() >= 2 && bytes[0..2] == [0xFF, 0xD8] {
        use zune_core::colorspace::ColorSpace;
        use zune_core::options::DecoderOptions;
        use zune_jpeg::JpegDecoder;

        let options = DecoderOptions::default()
            .jpeg_set_out_colorspace(ColorSpace::RGBA)
            .set_strict_mode(false);
        let mut decoder = JpegDecoder::new_with_options(bytes, options);
        decoder.decode_headers()?;
        let info = decoder.info().unwrap();
        let pixels = decoder.decode()?;
        Ok(RGBA8Buffer {
            width: info.width as _,
            height: info.height as _,
            data: pixels,
        })
    } else if bytes.len() >= 8 && bytes[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        use zune_core::options::DecoderOptions;
        use zune_png::PngDecoder;
        let options = DecoderOptions::default()
            .png_set_add_alpha_channel(true)
            .set_strict_mode(false);
        let mut decoder = PngDecoder::new_with_options(bytes, options);

        decoder.decode_headers().unwrap();
        let (width, height) = decoder.get_dimensions().unwrap();

        let pixels = decoder.decode_raw().unwrap();
        assert!(pixels.len() == width * height * 4); // and deal with u16 png later
        Ok(RGBA8Buffer {
            width: width as _,
            height: height as _,
            data: pixels,
        })
    } else {
        panic!()
    }
}
