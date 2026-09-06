//! Pixel-art loading and rendering for the terminal scene canvas.
//!
//! `.px` images deliberately have a tiny, strict grammar so malformed content
//! is found while developing an asset instead of becoming a broken scene.

use std::{fmt, str::FromStr};

use ratatui::{buffer::Buffer, layout::Rect, style::Color};

/// The six colors available to Pioneer Trail sprites.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pixel {
    Black,
    White,
    Green,
    Violet,
    Orange,
    Blue,
}

impl Pixel {
    /// Parses one palette character. `.` is transparent and is therefore `None`.
    pub const fn from_char(ch: char) -> Option<Option<Self>> {
        match ch {
            '.' => Some(None),
            'K' => Some(Some(Self::Black)),
            'W' => Some(Some(Self::White)),
            'G' => Some(Some(Self::Green)),
            'V' => Some(Some(Self::Violet)),
            'O' => Some(Some(Self::Orange)),
            'B' => Some(Some(Self::Blue)),
            _ => None,
        }
    }

    /// The source-file character for this palette entry.
    pub const fn as_char(self) -> char {
        match self {
            Self::Black => 'K',
            Self::White => 'W',
            Self::Green => 'G',
            Self::Violet => 'V',
            Self::Orange => 'O',
            Self::Blue => 'B',
        }
    }
}

/// A parsed `.px` image. Pixels are row-major and `None` means transparent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PxImage {
    width: u16,
    height: u16,
    pixels: Vec<Option<Pixel>>,
}

impl PxImage {
    /// Parses a `.px` source. Comment rows begin with `#`; blank rows are not valid pixels.
    pub fn parse(source: &str) -> Result<Self, PxError> {
        let rows: Vec<(usize, &str)> =
            source.lines().enumerate().filter(|(_, line)| !line.starts_with('#')).collect();
        if rows.is_empty() {
            return Err(PxError::Empty);
        }
        let width = rows[0].1.chars().count();
        if width == 0 {
            return Err(PxError::EmptyRow { line: rows[0].0 + 1 });
        }
        // This is intentionally far below u16::MAX: the renderer must never
        // allocate or iterate an unbounded canvas from an arbitrary file.
        if width > 256 || rows.len() > 256 {
            return Err(PxError::TooLarge);
        }

        let mut pixels = Vec::with_capacity(width * rows.len());
        for (line_index, row) in rows {
            let actual = row.chars().count();
            if actual == 0 {
                return Err(PxError::EmptyRow { line: line_index + 1 });
            }
            if actual != width {
                return Err(PxError::RaggedRow { line: line_index + 1, expected: width, actual });
            }
            for (column, ch) in row.chars().enumerate() {
                let pixel = Pixel::from_char(ch).ok_or(PxError::InvalidPixel {
                    line: line_index + 1,
                    column: column + 1,
                    found: ch,
                })?;
                pixels.push(pixel);
            }
        }
        Ok(Self { width: width as u16, height: (pixels.len() / width) as u16, pixels })
    }

    pub const fn width(&self) -> u16 {
        self.width
    }
    pub const fn height(&self) -> u16 {
        self.height
    }
    pub const fn cell_height(&self) -> u16 {
        self.height.div_ceil(2)
    }

    /// Returns the pixel at source coordinates, or `None` when outside the image.
    pub fn pixel(&self, x: u16, y: u16) -> Option<Pixel> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.pixels[(y as usize * self.width as usize) + x as usize]
    }
}

impl FromStr for PxImage {
    type Err = PxError;
    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Self::parse(source)
    }
}

/// Why a `.px` file was rejected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PxError {
    Empty,
    EmptyRow { line: usize },
    RaggedRow { line: usize, expected: usize, actual: usize },
    InvalidPixel { line: usize, column: usize, found: char },
    TooLarge,
}

impl fmt::Display for PxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "sprite has no pixel rows"),
            Self::EmptyRow { line } => write!(f, "empty pixel row at line {line}"),
            Self::RaggedRow { line, expected, actual } => {
                write!(f, "ragged row at line {line}: expected {expected} pixels, got {actual}")
            }
            Self::InvalidPixel { line, column, found } => {
                write!(f, "invalid pixel {found:?} at line {line}, column {column}")
            }
            Self::TooLarge => write!(f, "sprite dimensions exceed 65535 pixels"),
        }
    }
}

impl std::error::Error for PxError {}

/// Terminal palette selected by the user's color-mode setting.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ColorMode {
    #[default]
    TrueColor,
    Ansi256,
    Ansi16,
    Mono,
}

impl ColorMode {
    pub const fn color(self, pixel: Pixel) -> Color {
        match self {
            Self::TrueColor => match pixel {
                Pixel::Black => Color::Rgb(0, 0, 0),
                Pixel::White => Color::Rgb(255, 255, 255),
                Pixel::Green => Color::Rgb(27, 203, 1),
                Pixel::Violet => Color::Rgb(228, 52, 254),
                Pixel::Orange => Color::Rgb(242, 106, 0),
                Pixel::Blue => Color::Rgb(27, 154, 254),
            },
            Self::Ansi256 => match pixel {
                Pixel::Black => Color::Indexed(16),
                Pixel::White => Color::Indexed(231),
                Pixel::Green => Color::Indexed(40),
                Pixel::Violet => Color::Indexed(165),
                Pixel::Orange => Color::Indexed(202),
                Pixel::Blue => Color::Indexed(33),
            },
            Self::Ansi16 => match pixel {
                Pixel::Black => Color::Black,
                Pixel::White => Color::White,
                Pixel::Green => Color::Green,
                Pixel::Violet => Color::Magenta,
                Pixel::Orange => Color::LightYellow,
                Pixel::Blue => Color::Blue,
            },
            Self::Mono => Color::White,
        }
    }

    const fn mono_glyph(pixel: Pixel) -> &'static str {
        match pixel {
            Pixel::Black => " ",
            Pixel::Blue => "░",
            Pixel::Violet => "▒",
            Pixel::Green | Pixel::Orange => "▓",
            Pixel::White => "█",
        }
    }
}

/// Renders an image into terminal cells. Two vertical source pixels share one cell.
/// Transparent pixels preserve the existing cell, making layers composable.
/// Images are clipped to both `area` and the destination buffer.
pub fn render(image: &PxImage, buffer: &mut Buffer, area: Rect, mode: ColorMode) {
    let bounds = area.intersection(buffer.area);
    let columns = image.width.min(bounds.width);
    let rows = image.cell_height().min(bounds.height);
    for y in 0..rows {
        for x in 0..columns {
            let top = image.pixel(x, y * 2);
            let bottom = image.pixel(x, y * 2 + 1);
            let cell =
                buffer.cell_mut((bounds.x + x, bounds.y + y)).expect("clipped to buffer bounds");
            if mode == ColorMode::Mono {
                let glyph = match (top, bottom) {
                    (None, None) => continue,
                    (Some(top), Some(bottom)) => {
                        if mono_level(top) >= mono_level(bottom) {
                            ColorMode::mono_glyph(top)
                        } else {
                            ColorMode::mono_glyph(bottom)
                        }
                    }
                    (Some(pixel), None) | (None, Some(pixel)) => ColorMode::mono_glyph(pixel),
                };
                cell.set_symbol(glyph).set_fg(Color::White).set_bg(Color::Black);
                continue;
            }
            match (top, bottom) {
                (None, None) => continue,
                (Some(top), None) => {
                    cell.set_symbol("▀").set_fg(mode.color(top)).set_bg(Color::Black)
                }
                (None, Some(bottom)) => {
                    cell.set_symbol("▄").set_fg(mode.color(bottom)).set_bg(Color::Black)
                }
                (Some(top), Some(bottom)) => {
                    cell.set_symbol("▀").set_fg(mode.color(top)).set_bg(mode.color(bottom))
                }
            };
        }
    }
}

const fn mono_level(pixel: Pixel) -> u8 {
    match pixel {
        Pixel::Black => 0,
        Pixel::Blue => 1,
        Pixel::Violet => 2,
        Pixel::Green | Pixel::Orange => 3,
        Pixel::White => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{buffer::Buffer, layout::Rect, style::Color};

    #[test]
    fn parses_comments_and_transparency() {
        let image = PxImage::parse("# wagon\n.W\nGO\n").unwrap();
        assert_eq!((image.width(), image.height(), image.cell_height()), (2, 2, 1));
        assert_eq!(image.pixel(0, 0), None);
        assert_eq!(image.pixel(1, 1), Some(Pixel::Orange));
    }

    #[test]
    fn rejects_invalid_pixel() {
        assert_eq!(
            PxImage::parse("W?\nWW\n").unwrap_err(),
            PxError::InvalidPixel { line: 1, column: 2, found: '?' }
        );
    }

    #[test]
    fn rejects_ragged_and_blank_rows() {
        assert!(matches!(PxImage::parse("WW\nW\n"), Err(PxError::RaggedRow { .. })));
        assert!(matches!(PxImage::parse("WW\n\n"), Err(PxError::EmptyRow { .. })));
    }

    #[test]
    fn renders_half_blocks_with_palette_fallback() {
        let image = PxImage::parse("W\nO\n").unwrap();
        let mut buffer = Buffer::empty(Rect::new(0, 0, 1, 1));
        render(&image, &mut buffer, Rect::new(0, 0, 1, 1), ColorMode::Ansi256);
        let cell = buffer.cell((0, 0)).unwrap();
        assert_eq!(cell.symbol(), "▀");
        assert_eq!(cell.fg, Color::Indexed(231));
        assert_eq!(cell.bg, Color::Indexed(202));
    }

    #[test]
    fn mono_uses_luminance_glyphs() {
        let image = PxImage::parse("B\n").unwrap();
        let mut buffer = Buffer::empty(Rect::new(0, 0, 1, 1));
        render(&image, &mut buffer, Rect::new(0, 0, 1, 1), ColorMode::Mono);
        assert_eq!(buffer.cell((0, 0)).unwrap().symbol(), "░");
    }
}
