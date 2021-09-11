
use model::types::StaticVertex;

/// The number of glyphs stored in the texture file
const FONT_TEXTURE_GLYPH_COUNT: i32 = 128;

/// The size (width and height) of the texture file, in pixels
const FONT_TEXTURE_SIZE: f32 = 512.0;

/// Glyph struct
/// Properties of a glyph, as exported into the description file exported from Hiero
#[derive(Copy, Clone)]
struct Glyph {
    texture_s: f32,
    texture_t: f32,
    offset_x: f32,
    offset_y: f32,
    width: f32,
    height: f32,
    advance_x: f32
}

impl Glyph {

    /// Create a new instance with all fields set to zero
    fn new() -> Glyph {
        Glyph {
            texture_s: 0.0,
            texture_t: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            width: 1.0,
            height: 1.0,
            advance_x: 0.0
        }
    }
}

/// TextAlignment enum
/// The alignment options that can be specified along either the vertical or horizontal axis
pub enum TextAlignment {
    Start,
    Centre,
    End
}

/// TextGenerator struct
/// The full set of glyphs, and other information, decoded from a description file exported from
/// Hiero. Can produce vertex buffers for text rendering based on the glyph data that it holds.
pub struct TextGenerator {
    descent_to_baseline: f32,
    line_height: f32,
    glyphs: Vec<Glyph>
}

impl TextGenerator {

    /// Constant strings used internally when decoding the description file exported from Hiero
    const KEY_LINE_INFO: &'static str = "info";
    const KEY_LINE_COMMON: &'static str = "common";
    const KEY_LINE_PAGE: &'static str = "page";
    const KEY_LINE_CHARS: &'static str = "chars";
    const KEY_LINE_CHAR: &'static str = "char";
    const KEY_FIELD_LINE_HEIGHT: &'static str = "lineHeight";
    const KEY_FIELD_LINE_BASE: &'static str = "base";
    const KEY_FIELD_CHAR_COUNT: &'static str = "count";
    const KEY_FIELD_ID: &'static str = "id";
    const KEY_FIELD_X: &'static str = "x";
    const KEY_FIELD_Y: &'static str = "y";
    const KEY_FIELD_WIDTH: &'static str = "width";
    const KEY_FIELD_HEIGHT: &'static str = "height";
    const KEY_FIELD_OFFSET_X: &'static str = "xoffset";
    const KEY_FIELD_OFFSET_Y: &'static str = "yoffset";
    const KEY_FIELD_X_ADVANCE: &'static str = "xadvance";

    /// Build a new instance, initialised fully from file data
    pub fn from_resource(file_data: &str) -> TextGenerator {
        let mut glyph_set = vec!();
        glyph_set.resize(FONT_TEXTURE_GLYPH_COUNT as usize, Glyph::new());

        let mut base: Option<i32> = None;
        let mut line_height: Option<i32> = None;
        let mut char_count: Option<i32> = None;

        let mut id: Option<i32> = None;
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut width: i32 = 0;
        let mut height: i32 = 0;
        let mut offset_x: i32 = 0;
        let mut offset_y: i32 = 0;
        let mut x_advance: i32 = 0;

        // Iterate over all lines in file, check first word and handle from there
        for line in file_data.lines() {

            // Assemble glyph if just read from a line
            if let Some(pending_id) = id {
                if pending_id < FONT_TEXTURE_GLYPH_COUNT {
                    glyph_set[pending_id as usize] = Glyph {
                        texture_s: x as f32,
                        texture_t: y as f32,
                        width: width as f32,
                        height: height as f32,
                        offset_x: offset_x as f32,
                        offset_y: offset_y as f32,
                        advance_x: x_advance as f32
                    };
                }
                id = None;
            }

            // Read next line
            let mut word_iter = line.split_whitespace();
            let line_key = word_iter.next().unwrap();
            if line_key == Self::KEY_LINE_INFO {
                // Nothing to get from info line
            } else if line_key == Self::KEY_LINE_COMMON {
                // Get line height and base integers from common line
                while let Some(next_word) = word_iter.next() {
                    let sign_pos = match next_word.find('=') {
                        Some(pos) => pos,
                        None => continue
                    };
                    let value = next_word[(sign_pos + 1)..].parse::<i32>().unwrap();
                    if next_word.starts_with(Self::KEY_FIELD_LINE_HEIGHT) {
                        line_height = Some(value);
                    } else if next_word.starts_with(Self::KEY_FIELD_LINE_BASE) {
                        base = Some(value);
                    }
                }
            } else if line_key == Self::KEY_LINE_PAGE {
                // Nothing to get from page line
            } else if line_key == Self::KEY_LINE_CHARS {
                // Get count integer from chars line
                while let Some(next_word) = word_iter.next() {
                    if next_word.starts_with(Self::KEY_FIELD_CHAR_COUNT) {
                        let sign_pos = match next_word.find('=') {
                            Some(pos) => pos,
                            None => continue
                        };
                        char_count = Some(
                            next_word[(sign_pos + 1)..].parse::<i32>().unwrap());
                    }
                }
            } else if line_key == Self::KEY_LINE_CHAR {
                // Get all fields for this glyph, then add to glyph set
                while let Some(next_word) = word_iter.next() {
                    let sign_pos = match next_word.find('=') {
                        Some(pos) => pos,
                        None => continue
                    };
                    let value = next_word[(sign_pos + 1)..].parse::<i32>().unwrap();
                    if next_word.starts_with(Self::KEY_FIELD_ID) {
                        id = Some(value);
                    } else if next_word.starts_with(Self::KEY_FIELD_WIDTH) {
                        width = value;
                    } else if next_word.starts_with(Self::KEY_FIELD_HEIGHT) {
                        height = value;
                    } else if next_word.starts_with(Self::KEY_FIELD_OFFSET_X) {
                        offset_x = value;
                    } else if next_word.starts_with(Self::KEY_FIELD_OFFSET_Y) {
                        offset_y = value;
                    } else if next_word.starts_with(Self::KEY_FIELD_X_ADVANCE) {
                        x_advance = value;
                    } else if next_word.starts_with(Self::KEY_FIELD_X) {
                        x = value;
                    } else if next_word.starts_with(Self::KEY_FIELD_Y) {
                        y = value;
                    }
                }
            }
        }

        match (base, line_height, char_count) {
            (Some(b), Some(l), Some(_)) => TextGenerator {
                descent_to_baseline: b as f32,
                line_height: l as f32,
                glyphs: glyph_set
            },
            _ => panic!()
        }
    }

    /// Generate data for a vertex buffer. Vertices use the format specified by the
    /// model::types::StaticVertex struct, and there are 6 vertices per character.
    /// Text will be drawn from (left, top) in an area sized by (box_width, box_height).
    /// TODO - This comment does not describe how text fitting is affected by all parameters
    pub fn generate_vertex_buffer(
        &self,
        for_text: &str,
        left_top: (f32, f32),
        box_size: (f32, f32),
        max_line_height: f32,
        horizontal_alignment: TextAlignment,
        vertical_alignment: TextAlignment
    ) -> Vec<StaticVertex> {

        let text_chars: Vec<char> = for_text.chars().collect();
        let vertex_count = text_chars.len() * 6;
        let mut vertices: Vec<StaticVertex> = vec![StaticVertex::default(); vertex_count];

        let line_height_units = match box_size.1 < max_line_height {
            true => box_size.1,
            false => max_line_height
        };
        let units_per_font_pixel = line_height_units / self.line_height as f32;

        // First pass determines how many lines need to be rendered, and how many
        // characters will be on each of those lines
        let mut characters_per_line: Vec<usize> = vec![];
        let mut unit_width_of_line: Vec<f32> = vec![];
        let mut units_across_this_line: f32 = 0.0;
        let mut units_in_line_up_to_word_end: f32 = 0.0;
        let mut current_word_begun_at: usize = 0;
        let mut units_into_this_word: f32 = 0.0;
        let mut chars_for_this_line = 0;
        for (index, c) in for_text.char_indices() {
            let glyph = self.glyphs[c as usize];
            let advance = glyph.advance_x as f32 * units_per_font_pixel;
            units_across_this_line += advance;
            units_into_this_word += advance;
            chars_for_this_line += 1;
            if c == ' ' {
                units_in_line_up_to_word_end = units_across_this_line - advance;
                current_word_begun_at = index + 1;
                units_into_this_word = 0.0;
            } else if units_across_this_line > box_size.0 {
                if index - current_word_begun_at + 1 == chars_for_this_line {
                    characters_per_line.push(index - current_word_begun_at);
                    current_word_begun_at = index;
                    chars_for_this_line = 1;
                    unit_width_of_line.push(units_across_this_line - advance);
                    units_across_this_line = advance;
                    units_into_this_word = advance;
                } else {
                    let characters_for_next_line = index + 1 - current_word_begun_at;
                    characters_per_line.push(chars_for_this_line - characters_for_next_line);
                    chars_for_this_line = characters_for_next_line;
                    unit_width_of_line.push(units_in_line_up_to_word_end);
                    units_across_this_line = units_into_this_word;
                }
            }
        }
        if chars_for_this_line > 0 {
            characters_per_line.push(chars_for_this_line);
            unit_width_of_line.push(units_across_this_line);
        }

        // Set side margin, horizontal margin depends on supplied alignment
        let total_text_height_units = characters_per_line.len() as f32 * line_height_units;
        let margin_y_units: f32 = match vertical_alignment {
            TextAlignment::Start => 0.0,
            TextAlignment::Centre => 0.5 * (box_size.1 - total_text_height_units),
            TextAlignment::End => box_size.1 - total_text_height_units
        };

        // Start building the buffer
        let mut chars_rendered = 0;
        let mut pen_y =
            left_top.1 + margin_y_units + self.descent_to_baseline as f32 * units_per_font_pixel;
        let mut text_index: usize = 0;
        for (index, chars_on_line) in characters_per_line.iter().enumerate() {
            let line_width_units = unit_width_of_line[index];
            let margin_x_units: f32 = match horizontal_alignment {
                TextAlignment::Start => 0.0,
                TextAlignment::End => box_size.0 - line_width_units,
                TextAlignment::Centre => 0.5 * (box_size.0 - line_width_units)
            };
            let mut pen_x = left_top.0 + margin_x_units;
            for _i in 0..(*chars_on_line as i32) {
                let char = text_chars[text_index];
                text_index += 1;
                let glyph = self.glyphs[char as usize];

                let x_min = pen_x + glyph.offset_x as f32 * units_per_font_pixel;
                let x_max = x_min + glyph.width as f32 * units_per_font_pixel;
                let y_min = pen_y -
                    (self.descent_to_baseline - glyph.offset_y) as f32 * units_per_font_pixel;
                let y_max = y_min + glyph.height as f32 * units_per_font_pixel;

                let s_min = glyph.texture_s as f32 / FONT_TEXTURE_SIZE;
                let s_max = s_min + glyph.width as f32 / FONT_TEXTURE_SIZE;
                let t_min = glyph.texture_t as f32 / FONT_TEXTURE_SIZE;
                let t_max = t_min + glyph.height as f32 / FONT_TEXTURE_SIZE;

                let i = chars_rendered * 6;
                vertices[i    ] = StaticVertex::from_components(
                    (x_min, y_min, 0.0), (0.0, 0.0, -1.0), (s_min, t_min)
                );
                vertices[i + 1] = StaticVertex::from_components(
                    (x_min, y_max, 0.0), (0.0, 0.0, -1.0), (s_min, t_max)
                );
                vertices[i + 2] = StaticVertex::from_components(
                    (x_max, y_max, 0.0), (0.0, 0.0, -1.0), (s_max, t_max)
                );
                vertices[i + 3] = StaticVertex::from_components(
                    (x_max, y_max, 0.0), (0.0, 0.0, -1.0), (s_max, t_max)
                );
                vertices[i + 4] = StaticVertex::from_components(
                    (x_max, y_min, 0.0), (0.0, 0.0, -1.0), (s_max, t_min)
                );
                vertices[i + 5] = StaticVertex::from_components(
                    (x_min, y_min, 0.0), (0.0, 0.0, -1.0), (s_min, t_min)
                );

                pen_x += glyph.advance_x * units_per_font_pixel;
                chars_rendered += 1;
            }
            pen_y += line_height_units;
        }
        vertices
    }
}
