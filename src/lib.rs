#![crate_name="orbfont"]
#![crate_type="lib"]

extern crate orbclient;
extern crate rusttype;

#[cfg(not(target_os = "redox"))]
pub extern crate font_loader;
#[cfg(not(target_os = "redox"))]
pub use font_loader::system_fonts::FontPropertyBuilder;
#[cfg(not(target_os = "redox"))]
pub use font_loader::system_fonts;
#[cfg(not(target_os = "redox"))]
pub use font_loader::system_fonts::FontProperty;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use orbclient::{Color, Renderer};

pub struct Font {
    inner: rusttype::Font<'static>
}

impl Font {
    /// Find a font from an optional type, family, and style, such as "Mono", "Fira", "Regular"
    #[cfg(target_os = "redox")]
    pub fn find(typeface: Option<&str>, family: Option<&str>, style: Option<&str>) -> Result<Font, String> {
        Font::from_path(&format!("/ui/fonts/{}/{}/{}.ttf", typeface.unwrap_or("Mono"), family.unwrap_or("Fira"), style.unwrap_or("Regular")))
    }

    // A funciton to automate the process of building a font property  from "typeface, family, style"
    #[cfg(not(target_os = "redox"))]
    fn build_fontproperty (typeface: Option<&str>, family: Option<&str>, style: Option<&str>) -> FontProperty {
        let mut font = FontPropertyBuilder::new();
        if let Some(style) = style {
            let style_caps = &style.to_uppercase();
            let italic = style_caps.contains("ITALIC");
            let oblique = style_caps.contains("OBLIQUE");
            let bold = style_caps.contains("BOLD");
            if italic {
                font = font.italic();
            }
            if oblique {
                font = font.oblique();
            }
            if bold {
                font = font.bold();
            }
        }
        if let Some(typeface) = typeface {
            // FontProperty has no support for differentiating Sans and Serif.
            let typeface_caps = &typeface.to_uppercase();
            if typeface_caps.contains("MONO") {
                font = font.monospace();
            }
        }
        if let Some(family) = family {
            if let Some(typeface) = typeface {
                let typeface_caps = &typeface.to_uppercase();
                // manually adding Serif and Sans
                if typeface_caps.contains("SERIF") {
                    font = font.family(&[family, "Serif"].concat());
                } else if typeface_caps.contains("SANS") {
                    font = font.family(&[family, "Sans"].concat())
                }
            } else {
                font = font.family(family);
            }
        }
        font.build()
    }

    #[cfg(not(target_os = "redox"))]
    pub fn find(typeface: Option<&str>, family: Option<&str>, style: Option<&str>) -> Result<Font, String> {
        // This funciton attempts to use the rust-font-loader library, a frontend
        // to the ubiquitous C library fontconfig, to find and load the specified
        // font.
        let mut font = Font::build_fontproperty(typeface, family, style);
        // font_loader::query specific returns an empty vector if there are no matches
        // and does not tag the result with associated data like "italic", merely returns
        // the name of the font if it exists.
        let fonts = system_fonts::query_specific(&mut font); // Returns an empty vector if there are no matches.
        // Confirm that a font matched:
        if fonts.len() >= 1 {
            // get the matched font straight from the data:
            let font_data = system_fonts::get(&font); // Getting font data from properties
            match font_data {
                Some((data, _)) => Ok(Font::from_data(data.into_boxed_slice())?),
                None => Err(format!("Could not get font {} from data", &fonts[0]))
            }
        } else {
            // If no font matched, try again with no family, as concatenating "Sans" or "Serif" may rule out legitimate fonts
            let mut font = Font::build_fontproperty(None, family, style);
            let fonts = system_fonts::query_specific(&mut font);
            if fonts.len() >= 1 {
                let font_data = system_fonts::get(&font);
                match font_data {
                    Some((data, _)) => Ok(Font::from_data(data.into_boxed_slice())?),
                    None => Err(format!("Could not get font {} from data", &fonts[0]))
                }
            }  else {
                // If no font matched, try to load the default font manually
                Font::from_path("/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf")
            }
        }
    }

    /// Load a font from file path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Font, String> {
        let mut file = try!(File::open(path).map_err(|err| format!("failed to open font: {}", err)));
        let mut data = Vec::new();
        let _ = try!(file.read_to_end(&mut data).map_err(|err| format!("failed to read font: {}", err)));
        Font::from_data(data)
    }

    /// Load a font from a slice
    pub fn from_data<D: Into<rusttype::SharedBytes<'static>>>(data: D) -> Result<Font, String> {
        if let Ok(collection) = rusttype::FontCollection::from_bytes(data) {
            if let Ok(font) = collection.into_font() {
                return Ok(Font {
                    inner: font
                })
            } else {
                return Err("error constructing a FontCollection from bytes".to_string());
            }
        } else {
            return Err("font collection did not have exactly one font".to_string())
        }
    }

    /// Render provided text using the font
    pub fn render<'a>(&'a self, text: &str, height: f32) -> Text<'a> {
        let scale = rusttype::Scale::uniform(height);

        // The origin of a line of text is at the baseline (roughly where non-descending letters sit).
        // We don't want to clip the text, so we shift it down with an offset when laying it out.
        // v_metrics.ascent is the distance between the baseline and the highest edge of any glyph in
        // the font. That's enough to guarantee that there's no clipping.
        let v_metrics = self.inner.v_metrics(scale);
        let offset = rusttype::point(0.0, v_metrics.ascent);

        // Glyphs to draw for "RustType". Feel free to try other strings.
        let glyphs: Vec<rusttype::PositionedGlyph> = self.inner.layout(text, scale, offset).collect();

        // Find the most visually pleasing width to display
        let width = glyphs.iter().rev()
            .filter_map(|g| g.pixel_bounding_box()
                        .map(|b| b.min.x as f32 + g.unpositioned().h_metrics().advance_width))
            .next().unwrap_or(0.0);

        Text {
            w: width.ceil() as u32,
            h: height.ceil() as u32,
            glyphs: glyphs
        }
    }
}

pub struct Text<'a> {
    w: u32,
    h: u32,
    glyphs: Vec<rusttype::PositionedGlyph<'a>>
}

impl<'a> Text<'a> {
    /// Return width of the text
    pub fn width(&self) -> u32 {
        self.w
    }

    /// Return height of the text
    pub fn height(&self) -> u32 {
        self.h
    }

    /// Draw the text onto a window and clipp the text to the given bounds
    pub fn draw_clipped<R: Renderer + ?Sized>(&self, renderer: &mut R, x: i32, y: i32, bounds_x: i32, bounds_width: u32, color: Color) {
        for g in self.glyphs.iter() {
            if let Some(bb) = g.pixel_bounding_box() {
                g.draw(|off_x, off_y, v| {
                    let off_x = off_x as i32 + bb.min.x;
                    let off_y = off_y as i32 + bb.min.y;
                    // There's still a possibility that the glyph clips the boundaries of the bitmap
                    if off_x >= 0 && off_x < self.w as i32 && off_y >= 0 && off_y < self.h as i32
                    && x + off_x >= bounds_x && x + off_x <= bounds_x + bounds_width as i32 {
                        let c = (v * 255.0) as u32;
                        renderer.pixel(x + off_x, y + off_y, Color{
                            data: c << 24 | (color.data & 0xFFFFFF)
                        });
                    }
                });
            }
        }
    }

    /// Draw the text onto a window
    pub fn draw<R: Renderer + ?Sized>(&self, renderer: &mut R, x: i32, y: i32, color: Color) {
       self.draw_clipped(renderer, x, y, x, self.w, color)
    }  
}
