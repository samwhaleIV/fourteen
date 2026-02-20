use super::{
    EngineTextures,
    TextureFrame,
    pipelines::text_pipeline::{
        FontDefinition,
        GlyphArea
    }
};

pub struct FontClassic;
pub struct FontClassicOutlined;
pub struct FontTwelven;
pub struct FontTwelvenShaded;

impl FontDefinition for FontClassic {
    fn get_texture(textures: &EngineTextures) -> TextureFrame {
        textures.font_classic
        .unwrap_or(textures.missing)
    }

    fn get_glyph(character: char) -> GlyphArea {
        match character {
            '!' => GlyphArea {x: 68,y: 38,width: 4,height: 10,y_offset: 0},
            '"' => GlyphArea {x: 100,y: 25,width: 5,height: 12,y_offset: -1},
            '%' => GlyphArea {x: 38,y: 38,width: 8,height: 10,y_offset: 0},
            '\'' => GlyphArea {x: 88,y: 25,width: 2,height: 12,y_offset: -1},
            '(' => GlyphArea {x: 48,y: 38,width: 3,height: 10,y_offset: 0},
            ')' => GlyphArea {x: 53,y: 38,width: 3,height: 10,y_offset: 0},
            '*' => GlyphArea {x: 82,y: 38,width: 4,height: 10,y_offset: 0},
            '+' => GlyphArea {x: 14,y: 38,width: 6,height: 10,y_offset: 0},
            ',' => GlyphArea {x: 84,y: 25,width: 2,height: 12,y_offset: -1},
            '-' => GlyphArea {x: 22,y: 38,width: 6,height: 10,y_offset: 0},
            '.' => GlyphArea {x: 80,y: 25,width: 2,height: 12,y_offset: -1},
            '/' => GlyphArea {x: 2,y: 38,width: 4,height: 10,y_offset: 0},
            '0' => GlyphArea {x: 2,y: 26,width: 5,height: 10,y_offset: 0},
            '1' => GlyphArea {x: 9,y: 26,width: 4,height: 10,y_offset: 0},
            '2' => GlyphArea {x: 15,y: 26,width: 6,height: 10,y_offset: 0},
            '3' => GlyphArea {x: 23,y: 26,width: 6,height: 10,y_offset: 0},
            '4' => GlyphArea {x: 31,y: 26,width: 7,height: 10,y_offset: 0},
            '5' => GlyphArea {x: 40,y: 26,width: 6,height: 10,y_offset: 0},
            '6' => GlyphArea {x: 48,y: 26,width: 6,height: 10,y_offset: 0},
            '7' => GlyphArea {x: 56,y: 26,width: 6,height: 10,y_offset: 0},
            '8' => GlyphArea {x: 64,y: 26,width: 6,height: 10,y_offset: 0},
            '9' => GlyphArea {x: 72,y: 26,width: 6,height: 10,y_offset: 0},
            ':' => GlyphArea {x: 92,y: 25,width: 2,height: 12,y_offset: -1},
            ';' => GlyphArea {x: 96,y: 25,width: 2,height: 12,y_offset: -1},
            '<' => GlyphArea {x: 88,y: 38,width: 4,height: 10,y_offset: 0},
            '=' => GlyphArea {x: 30,y: 38,width: 6,height: 10,y_offset: 0},
            '>' => GlyphArea {x: 94,y: 38,width: 4,height: 10,y_offset: 0},
            '?' => GlyphArea {x: 74,y: 38,width: 6,height: 10,y_offset: 0},
            '[' => GlyphArea {x: 58,y: 38,width: 3,height: 10,y_offset: 0},
            '\\' => GlyphArea {x: 8,y: 38,width: 4,height: 10,y_offset: 0},
            ']' => GlyphArea {x: 63,y: 38,width: 3,height: 10,y_offset: 0},
            '_' => GlyphArea {x: 100,y: 38,width: 6,height: 10,y_offset: 0},
            'a' | 'A' => GlyphArea {x: 2,y: 2,width: 6,height: 10,y_offset: 0},
            'b' | 'B' => GlyphArea {x: 10,y: 2,width: 6,height: 10,y_offset: 0},
            'c' | 'C' => GlyphArea {x: 18,y: 2,width: 5,height: 10,y_offset: 0},
            'd' | 'D' => GlyphArea {x: 25,y: 2,width: 6,height: 10,y_offset: 0},
            'e' | 'E' => GlyphArea {x: 33,y: 2,width: 4,height: 10,y_offset: 0},
            'f' | 'F' => GlyphArea {x: 39,y: 2,width: 4,height: 10,y_offset: 0},
            'g' | 'G' => GlyphArea {x: 45,y: 2,width: 6,height: 10,y_offset: 0},
            'h' | 'H' => GlyphArea {x: 53,y: 2,width: 6,height: 10,y_offset: 0},
            'i' | 'I' => GlyphArea {x: 61,y: 2,width: 4,height: 10,y_offset: 0},
            'j' | 'J' => GlyphArea {x: 67,y: 2,width: 4,height: 10,y_offset: 0},
            'k' | 'K' => GlyphArea {x: 73,y: 2,width: 6,height: 10,y_offset: 0},
            'l' | 'L' => GlyphArea {x: 81,y: 2,width: 3,height: 10,y_offset: 0},
            'm' | 'M' => GlyphArea {x: 86,y: 2,width: 10,height: 10,y_offset: 0},
            'n' | 'N' => GlyphArea {x: 98,y: 2,width: 6,height: 10,y_offset: 0},
            'o' | 'O' => GlyphArea {x: 2,y: 14,width: 6,height: 10,y_offset: 0},
            'p' | 'P' => GlyphArea {x: 10,y: 14,width: 6,height: 10,y_offset: 0},
            'q' | 'Q' => GlyphArea {x: 18,y: 14,width: 7,height: 10,y_offset: 0},
            'r' | 'R' => GlyphArea {x: 27,y: 14,width: 6,height: 10,y_offset: 0},
            's' | 'S' => GlyphArea {x: 35,y: 14,width: 5,height: 10,y_offset: 0},
            't' | 'T' => GlyphArea {x: 42,y: 14,width: 4,height: 10,y_offset: 0},
            'u' | 'U' => GlyphArea {x: 48,y: 14,width: 6,height: 10,y_offset: 0},
            'v' | 'V' => GlyphArea {x: 56,y: 14,width: 6,height: 10,y_offset: 0},
            'w' | 'W' => GlyphArea {x: 64,y: 14,width: 10,height: 10,y_offset: 0},
            'x' | 'X' => GlyphArea {x: 76,y: 14,width: 6,height: 10,y_offset: 0},
            'y' | 'Y' => GlyphArea {x: 84,y: 14,width: 6,height: 10,y_offset: 0},
            'z' | 'Z' => GlyphArea {x: 92,y: 14,width: 6,height: 10,y_offset: 0},
            _ => Default::default()
        }
    }

    const LINE_HEIGHT: f32 = 10.0;
    const LETTER_SPACING: f32 = 0.75;
    const WORD_SPACING: f32 = 3.0;
}

impl FontDefinition for FontClassicOutlined {
    fn get_texture(textures: &EngineTextures) -> TextureFrame {
        textures.font_classic_outline
        .unwrap_or(textures.missing)
    }
    fn get_glyph(character: char) -> GlyphArea {
        return match character {
            '!' => GlyphArea {x: 135,y: 75,width: 10,height: 22,y_offset: 0},
            '"' => GlyphArea {x: 199,y: 49,width: 12,height: 26,y_offset: -1},
            '%' => GlyphArea {x: 75,y: 75,width: 18,height: 22,y_offset: 0},
            '\'' => GlyphArea {x: 175,y: 49,width: 6,height: 26,y_offset: -1},
            '(' => GlyphArea {x: 95,y: 75,width: 8,height: 22,y_offset: 0},
            ')' => GlyphArea {x: 105,y: 75,width: 8,height: 22,y_offset: 0},
            '*' => GlyphArea {x: 163,y: 75,width: 10,height: 22,y_offset: 0},
            '+' => GlyphArea {x: 27,y: 75,width: 14,height: 22,y_offset: 0},
            ',' => GlyphArea {x: 167,y: 49,width: 6,height: 26,y_offset: -1},
            '-' => GlyphArea {x: 43,y: 75,width: 14,height: 22,y_offset: 0},
            '.' => GlyphArea {x: 159,y: 49,width: 6,height: 26,y_offset: -1},
            '/' => GlyphArea {x: 3,y: 75,width: 10,height: 22,y_offset: 0},
            '0' => GlyphArea {x: 3,y: 51,width: 14,height: 22,y_offset: 0},
            '1' => GlyphArea {x: 19,y: 51,width: 10,height: 22,y_offset: 0},
            '2' => GlyphArea {x: 31,y: 51,width: 14,height: 22,y_offset: 0},
            '3' => GlyphArea {x: 47,y: 51,width: 14,height: 22,y_offset: 0},
            '4' => GlyphArea {x: 63,y: 51,width: 16,height: 22,y_offset: 0},
            '5' => GlyphArea {x: 81,y: 51,width: 14,height: 22,y_offset: 0},
            '6' => GlyphArea {x: 97,y: 51,width: 14,height: 22,y_offset: 0},
            '7' => GlyphArea {x: 113,y: 51,width: 14,height: 22,y_offset: 0},
            '8' => GlyphArea {x: 129,y: 51,width: 14,height: 22,y_offset: 0},
            '9' => GlyphArea {x: 145,y: 51,width: 14,height: 22,y_offset: 0},
            ':' => GlyphArea {x: 183,y: 49,width: 6,height: 26,y_offset: -1},
            ';' => GlyphArea {x: 191,y: 49,width: 6,height: 26,y_offset: -1},
            '<' => GlyphArea {x: 175,y: 75,width: 10,height: 22,y_offset: 0},
            '=' => GlyphArea {x: 59,y: 75,width: 14,height: 22,y_offset: 0},
            '>' => GlyphArea {x: 187,y: 75,width: 10,height: 22,y_offset: 0},
            '?' => GlyphArea {x: 147,y: 75,width: 14,height: 22,y_offset: 0},
            '[' => GlyphArea {x: 115,y: 75,width: 8,height: 22,y_offset: 0},
            '\\' => GlyphArea {x: 15,y: 75,width: 10,height: 22,y_offset: 0},
            ']' => GlyphArea {x: 125,y: 75,width: 8,height: 22,y_offset: 0},
            'a' | 'A' => GlyphArea {x: 3,y: 3,width: 14,height: 22,y_offset: 0},
            'b' | 'B' => GlyphArea {x: 19,y: 3,width: 14,height: 22,y_offset: 0},
            'c' | 'C' => GlyphArea {x: 35,y: 3,width: 12,height: 22,y_offset: 0},
            'd' | 'D' => GlyphArea {x: 49,y: 3,width: 14,height: 22,y_offset: 0},
            'e' | 'E' => GlyphArea {x: 65,y: 3,width: 12,height: 22,y_offset: 0},
            'f' | 'F' => GlyphArea {x: 79,y: 3,width: 12,height: 22,y_offset: 0},
            'g' | 'G' => GlyphArea {x: 93,y: 3,width: 14,height: 22,y_offset: 0},
            'h' | 'H' => GlyphArea {x: 109,y: 3,width: 14,height: 22,y_offset: 0},
            'i' | 'I' => GlyphArea {x: 125,y: 3,width: 10,height: 22,y_offset: 0},
            'j' | 'J' => GlyphArea {x: 137,y: 3,width: 10,height: 22,y_offset: 0},
            'k' | 'K' => GlyphArea {x: 149,y: 3,width: 14,height: 22,y_offset: 0},
            'l' | 'L' => GlyphArea {x: 165,y: 3,width: 10,height: 22,y_offset: 0},
            'm' | 'M' => GlyphArea {x: 177,y: 3,width: 22,height: 22,y_offset: 0},
            'n' | 'N' => GlyphArea {x: 201,y: 3,width: 14,height: 22,y_offset: 0},
            'o' | 'O' => GlyphArea {x: 3,y: 27,width: 14,height: 22,y_offset: 0},
            'p' | 'P' => GlyphArea {x: 19,y: 27,width: 14,height: 22,y_offset: 0},
            'q' | 'Q' => GlyphArea {x: 35,y: 27,width: 16,height: 22,y_offset: 0},
            'r' | 'R' => GlyphArea {x: 53,y: 27,width: 14,height: 22,y_offset: 0},
            's' | 'S' => GlyphArea {x: 69,y: 27,width: 12,height: 22,y_offset: 0},
            't' | 'T' => GlyphArea {x: 83,y: 27,width: 14,height: 22,y_offset: 0},
            'u' | 'U' => GlyphArea {x: 99,y: 27,width: 14,height: 22,y_offset: 0},
            'v' | 'V' => GlyphArea {x: 115,y: 27,width: 14,height: 22,y_offset: 0},
            'w' | 'W' => GlyphArea {x: 131,y: 27,width: 22,height: 22,y_offset: 0},
            'x' | 'X' => GlyphArea {x: 155,y: 27,width: 14,height: 22,y_offset: 0},
            'y' | 'Y' => GlyphArea {x: 171,y: 27,width: 14,height: 22,y_offset: 0},
            'z' | 'Z' => GlyphArea {x: 187,y: 27,width: 14,height: 22,y_offset: 0},
            _ => Default::default()
        }
    }
    const LINE_HEIGHT: f32 = 22.0;
    const LETTER_SPACING: f32 = 1.0;
    const WORD_SPACING: f32 = 4.0;
}

fn get_twelven_glyph_area(character: char) -> GlyphArea {
    return match character {
        '!' => GlyphArea {x: 405,y: 38,width: 6,height: 34,y_offset: 0},
        '"' => GlyphArea {x: 649,y: 38,width: 9,height: 8,y_offset: 0},
        '%' => GlyphArea {x: 604,y: 45,width: 12,height: 20,y_offset: 7},
        '\'' => GlyphArea {x: 644,y: 38,width: 3,height: 8,y_offset: 0},
        '(' => GlyphArea {x: 433,y: 38,width: 10,height: 34,y_offset: 0},
        ')' => GlyphArea {x: 445,y: 38,width: 10,height: 34,y_offset: 0},
        '*' => GlyphArea {x: 591,y: 45,width: 11,height: 20,y_offset: 7},
        '+' => GlyphArea {x: 509,y: 45,width: 14,height: 20,y_offset: 7},
        ',' => GlyphArea {x: 618,y: 67,width: 5,height: 8,y_offset: 29},
        '-' => GlyphArea {x: 525,y: 45,width: 14,height: 20,y_offset: 7},
        '.' => GlyphArea {x: 625,y: 67,width: 4,height: 8,y_offset: 29},
        '/' => GlyphArea {x: 481,y: 38,width: 12,height: 34,y_offset: 0},
        '0' => GlyphArea {x: 692,y: 2,width: 18,height: 34,y_offset: 0},
        '1' => GlyphArea {x: 513,y: 2,width: 15,height: 34,y_offset: 0},
        '2' => GlyphArea {x: 530,y: 2,width: 18,height: 34,y_offset: 0},
        '3' => GlyphArea {x: 550,y: 2,width: 18,height: 34,y_offset: 0},
        '4' => GlyphArea {x: 570,y: 2,width: 20,height: 34,y_offset: 0},
        '5' => GlyphArea {x: 592,y: 2,width: 18,height: 34,y_offset: 0},
        '6' => GlyphArea {x: 612,y: 2,width: 18,height: 34,y_offset: 0},
        '7' => GlyphArea {x: 632,y: 2,width: 18,height: 34,y_offset: 0},
        '8' => GlyphArea {x: 652,y: 2,width: 18,height: 34,y_offset: 0},
        '9' => GlyphArea {x: 672,y: 2,width: 18,height: 34,y_offset: 0},
        ':' => GlyphArea {x: 631,y: 49,width: 4,height: 15,y_offset: 11},
        ';' => GlyphArea {x: 637,y: 49,width: 5,height: 15,y_offset: 11},
        '<' => GlyphArea {x: 557,y: 45,width: 8,height: 20,y_offset: 7},
        '=' => GlyphArea {x: 541,y: 45,width: 14,height: 20,y_offset: 7},
        '>' => GlyphArea {x: 567,y: 45,width: 8,height: 20,y_offset: 7},
        '?' => GlyphArea {x: 413,y: 38,width: 18,height: 34,y_offset: 0},
        'A' => GlyphArea {x: 2,y: 2,width: 21,height: 34,y_offset: 0},
        'B' => GlyphArea {x: 25,y: 2,width: 18,height: 34,y_offset: 0},
        'C' => GlyphArea {x: 45,y: 2,width: 18,height: 34,y_offset: 0},
        'D' => GlyphArea {x: 65,y: 2,width: 18,height: 34,y_offset: 0},
        'E' => GlyphArea {x: 85,y: 2,width: 14,height: 34,y_offset: 0},
        'F' => GlyphArea {x: 101,y: 2,width: 14,height: 34,y_offset: 0},
        'G' => GlyphArea {x: 117,y: 2,width: 21,height: 34,y_offset: 0},
        'H' => GlyphArea {x: 140,y: 2,width: 18,height: 34,y_offset: 0},
        'I' => GlyphArea {x: 160,y: 2,width: 10,height: 34,y_offset: 0},
        'J' => GlyphArea {x: 172,y: 2,width: 15,height: 34,y_offset: 0},
        'K' => GlyphArea {x: 189,y: 2,width: 15,height: 34,y_offset: 0},
        'L' => GlyphArea {x: 206,y: 2,width: 10,height: 34,y_offset: 0},
        'M' => GlyphArea {x: 218,y: 2,width: 23,height: 34,y_offset: 0},
        'N' => GlyphArea {x: 243,y: 2,width: 18,height: 34,y_offset: 0},
        'O' => GlyphArea {x: 263,y: 2,width: 18,height: 34,y_offset: 0},
        'P' => GlyphArea {x: 283,y: 2,width: 16,height: 34,y_offset: 0},
        'Q' => GlyphArea {x: 301,y: 2,width: 18,height: 34,y_offset: 0},
        'R' => GlyphArea {x: 321,y: 2,width: 18,height: 34,y_offset: 0},
        'S' => GlyphArea {x: 341,y: 2,width: 18,height: 34,y_offset: 0},
        'T' => GlyphArea {x: 361,y: 2,width: 18,height: 34,y_offset: 0},
        'U' => GlyphArea {x: 381,y: 2,width: 20,height: 34,y_offset: 0},
        'V' => GlyphArea {x: 403,y: 2,width: 18,height: 34,y_offset: 0},
        'W' => GlyphArea {x: 423,y: 2,width: 28,height: 34,y_offset: 0},
        'X' => GlyphArea {x: 453,y: 2,width: 18,height: 34,y_offset: 0},
        'Y' => GlyphArea {x: 473,y: 2,width: 18,height: 34,y_offset: 0},
        'Z' => GlyphArea {x: 493,y: 2,width: 18,height: 34,y_offset: 0},
        '[' => GlyphArea {x: 457,y: 38,width: 10,height: 34,y_offset: 0},
        '\\' => GlyphArea {x: 495,y: 38,width: 12,height: 34,y_offset: 0},
        ']' => GlyphArea {x: 469,y: 38,width: 10,height: 34,y_offset: 0},
        'a' => GlyphArea {x: 2,y: 53,width: 16,height: 19,y_offset: 15},
        'b' => GlyphArea {x: 238,y: 38,width: 14,height: 34,y_offset: 0},
        'c' => GlyphArea {x: 20,y: 53,width: 12,height: 19,y_offset: 15},
        'd' => GlyphArea {x: 254,y: 38,width: 14,height: 34,y_offset: 0},
        'e' => GlyphArea {x: 34,y: 53,width: 16,height: 19,y_offset: 15},
        'f' => GlyphArea {x: 270,y: 38,width: 15,height: 34,y_offset: 0},
        'g' => GlyphArea {x: 323,y: 53,width: 14,height: 30,y_offset: 15},
        'h' => GlyphArea {x: 307,y: 38,width: 14,height: 34,y_offset: 0},
        'i' => GlyphArea {x: 232,y: 47,width: 4,height: 25,y_offset: 9},
        'j' => GlyphArea {x: 389,y: 47,width: 14,height: 36,y_offset: 9},
        'k' => GlyphArea {x: 287,y: 38,width: 12,height: 34,y_offset: 0},
        'l' => GlyphArea {x: 301,y: 38,width: 4,height: 34,y_offset: 0},
        'm' => GlyphArea {x: 52,y: 53,width: 21,height: 19,y_offset: 15},
        'n' => GlyphArea {x: 75,y: 53,width: 13,height: 19,y_offset: 15},
        'o' => GlyphArea {x: 90,y: 53,width: 14,height: 19,y_offset: 15},
        'p' => GlyphArea {x: 339,y: 53,width: 14,height: 30,y_offset: 15},
        'q' => GlyphArea {x: 355,y: 53,width: 16,height: 30,y_offset: 15},
        'r' => GlyphArea {x: 106,y: 53,width: 12,height: 19,y_offset: 15},
        's' => GlyphArea {x: 120,y: 53,width: 12,height: 19,y_offset: 15},
        't' => GlyphArea {x: 218,y: 45,width: 12,height: 27,y_offset: 7},
        'u' => GlyphArea {x: 134,y: 53,width: 13,height: 19,y_offset: 15},
        'v' => GlyphArea {x: 149,y: 53,width: 14,height: 19,y_offset: 15},
        'w' => GlyphArea {x: 165,y: 53,width: 20,height: 19,y_offset: 15},
        'x' => GlyphArea {x: 187,y: 53,width: 13,height: 19,y_offset: 15},
        'y' => GlyphArea {x: 373,y: 53,width: 14,height: 30,y_offset: 15},
        'z' => GlyphArea {x: 202,y: 53,width: 14,height: 19,y_offset: 15},
        '|' => GlyphArea {x: 577,y: 45,width: 12,height: 20,y_offset: 7},
        _ => Default::default()
    }
}

impl FontDefinition for FontTwelven {
    fn get_texture(textures: &EngineTextures) -> TextureFrame {
        textures.font_twelven
        .unwrap_or(textures.missing)
    }
    fn get_glyph(character: char) -> GlyphArea {
        get_twelven_glyph_area(character)
    }
    const LINE_HEIGHT: f32 = 34.0;
    const LETTER_SPACING: f32 = 1.0;
    const WORD_SPACING: f32 = 8.0;
}

impl FontDefinition for FontTwelvenShaded {
    fn get_texture(textures: &EngineTextures) -> TextureFrame {
        textures.font_twelven_shaded
        .unwrap_or(textures.missing)
    }
    fn get_glyph(character: char) -> GlyphArea {
        get_twelven_glyph_area(character)
    }
    const LINE_HEIGHT: f32 = 34.0;
    const LETTER_SPACING: f32 = 1.0;
    const WORD_SPACING: f32 = 8.0;
}
