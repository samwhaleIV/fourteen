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

pub struct FontMonoElf;

macro_rules! area {
    ($x:expr, $y:expr, $w:expr, $h:expr, $yo:expr) => {
        GlyphArea {
            x: $x,
            y: $y,
            width: $w,
            height: $h,
            y_offset: $yo,
        }
    };
}

impl FontDefinition for FontClassic {
    fn get_texture(textures: &EngineTextures) -> TextureFrame {
        textures.font_classic
    }

    fn get_glyph(character: char) -> GlyphArea {
        match character {
            '!' =>          area!(68,38,4,10,0),
            '"' =>          area!(100,25,5,12,-1),
            '%' =>          area!(38,38,8,10,0),
            '\'' =>         area!(88,25,2,12,-1),
            '(' =>          area!(48,38,3,10,0),
            ')' =>          area!(53,38,3,10,0),
            '*' =>          area!(82,38,4,10,0),
            '+' =>          area!(14,38,6,10,0),
            ',' =>          area!(84,25,2,12,-1),
            '-' =>          area!(22,38,6,10,0),
            '.' =>          area!(80,25,2,12,-1),
            '/' =>          area!(2,38,4,10,0),
            '0' =>          area!(2,26,5,10,0),
            '1' =>          area!(9,26,4,10,0),
            '2' =>          area!(15,26,6,10,0),
            '3' =>          area!(23,26,6,10,0),
            '4' =>          area!(31,26,7,10,0),
            '5' =>          area!(40,26,6,10,0),
            '6' =>          area!(48,26,6,10,0),
            '7' =>          area!(56,26,6,10,0),
            '8' =>          area!(64,26,6,10,0),
            '9' =>          area!(72,26,6,10,0),
            ':' =>          area!(92,25,2,12,-1),
            ';' =>          area!(96,25,2,12,-1),
            '<' =>          area!(88,38,4,10,0),
            '=' =>          area!(30,38,6,10,0),
            '>' =>          area!(94,38,4,10,0),
            '?' =>          area!(74,38,6,10,0),
            '[' =>          area!(58,38,3,10,0),
            '\\' =>         area!(8,38,4,10,0),
            ']' =>          area!(63,38,3,10,0),
            '_' =>          area!(100,38,6,10,0),
            'a' | 'A' =>    area!(2,2,6,10,0),
            'b' | 'B' =>    area!(10,2,6,10,0),
            'c' | 'C' =>    area!(18,2,5,10,0),
            'd' | 'D' =>    area!(25,2,6,10,0),
            'e' | 'E' =>    area!(33,2,4,10,0),
            'f' | 'F' =>    area!(39,2,4,10,0),
            'g' | 'G' =>    area!(45,2,6,10,0),
            'h' | 'H' =>    area!(53,2,6,10,0),
            'i' | 'I' =>    area!(61,2,4,10,0),
            'j' | 'J' =>    area!(67,2,4,10,0),
            'k' | 'K' =>    area!(73,2,6,10,0),
            'l' | 'L' =>    area!(81,2,3,10,0),
            'm' | 'M' =>    area!(86,2,10,10,0),
            'n' | 'N' =>    area!(98,2,6,10,0),
            'o' | 'O' =>    area!(2,14,6,10,0),
            'p' | 'P' =>    area!(10,14,6,10,0),
            'q' | 'Q' =>    area!(18,14,7,10,0),
            'r' | 'R' =>    area!(27,14,6,10,0),
            's' | 'S' =>    area!(35,14,5,10,0),
            't' | 'T' =>    area!(42,14,4,10,0),
            'u' | 'U' =>    area!(48,14,6,10,0),
            'v' | 'V' =>    area!(56,14,6,10,0),
            'w' | 'W' =>    area!(64,14,10,10,0),
            'x' | 'X' =>    area!(76,14,6,10,0),
            'y' | 'Y' =>    area!(84,14,6,10,0),
            'z' | 'Z' =>    area!(92,14,6,10,0),
            _ => Default::default()
        }
    }

    const LINE_HEIGHT: f32 = 10.0;
    const LETTER_SPACING: f32 = 0.75;
    const WORD_SPACING: f32 = 2.0;
}

impl FontDefinition for FontClassicOutlined {
    fn get_texture(textures: &EngineTextures) -> TextureFrame {
        textures.font_classic_outline
    }
    fn get_glyph(character: char) -> GlyphArea {
        return match character {
            '!' =>          area!(135,75,10,22,0),
            '"' =>          area!(199,49,12,26,-1),
            '%' =>          area!(75,75,18,22,0),
            '\'' =>         area!(175,49,6,26,-1),
            '(' =>          area!(95,75,8,22,0),
            ')' =>          area!(105,75,8,22,0),
            '*' =>          area!(163,75,10,22,0),
            '+' =>          area!(27,75,14,22,0),
            ',' =>          area!(167,49,6,26,-1),
            '-' =>          area!(43,75,14,22,0),
            '.' =>          area!(159,49,6,26,-1),
            '/' =>          area!(3,75,10,22,0),
            '0' =>          area!(3,51,14,22,0),
            '1' =>          area!(19,51,10,22,0),
            '2' =>          area!(31,51,14,22,0),
            '3' =>          area!(47,51,14,22,0),
            '4' =>          area!(63,51,16,22,0),
            '5' =>          area!(81,51,14,22,0),
            '6' =>          area!(97,51,14,22,0),
            '7' =>          area!(113,51,14,22,0),
            '8' =>          area!(129,51,14,22,0),
            '9' =>          area!(145,51,14,22,0),
            ':' =>          area!(183,49,6,26,-1),
            ';' =>          area!(191,49,6,26,-1),
            '<' =>          area!(175,75,10,22,0),
            '=' =>          area!(59,75,14,22,0),
            '>' =>          area!(187,75,10,22,0),
            '?' =>          area!(147,75,14,22,0),
            '[' =>          area!(115,75,8,22,0),
            '\\' =>         area!(15,75,10,22,0),
            ']' =>          area!(125,75,8,22,0),
            'a' | 'A' =>    area!(3,3,14,22,0),
            'b' | 'B' =>    area!(19,3,14,22,0),
            'c' | 'C' =>    area!(35,3,12,22,0),
            'd' | 'D' =>    area!(49,3,14,22,0),
            'e' | 'E' =>    area!(65,3,12,22,0),
            'f' | 'F' =>    area!(79,3,12,22,0),
            'g' | 'G' =>    area!(93,3,14,22,0),
            'h' | 'H' =>    area!(109,3,14,22,0),
            'i' | 'I' =>    area!(125,3,10,22,0),
            'j' | 'J' =>    area!(137,3,10,22,0),
            'k' | 'K' =>    area!(149,3,14,22,0),
            'l' | 'L' =>    area!(165,3,10,22,0),
            'm' | 'M' =>    area!(177,3,22,22,0),
            'n' | 'N' =>    area!(201,3,14,22,0),
            'o' | 'O' =>    area!(3,27,14,22,0),
            'p' | 'P' =>    area!(19,27,14,22,0),
            'q' | 'Q' =>    area!(35,27,16,22,0),
            'r' | 'R' =>    area!(53,27,14,22,0),
            's' | 'S' =>    area!(69,27,12,22,0),
            't' | 'T' =>    area!(83,27,14,22,0),
            'u' | 'U' =>    area!(99,27,14,22,0),
            'v' | 'V' =>    area!(115,27,14,22,0),
            'w' | 'W' =>    area!(131,27,22,22,0),
            'x' | 'X' =>    area!(155,27,14,22,0),
            'y' | 'Y' =>    area!(171,27,14,22,0),
            'z' | 'Z' =>    area!(187,27,14,22,0),
            _ => Default::default()
        }
    }
    const LINE_HEIGHT: f32 = 22.0;
    const LETTER_SPACING: f32 = 1.0;
    const WORD_SPACING: f32 = 6.0;
}

fn get_twelven_glyph_area(character: char) -> GlyphArea {
    return match character {
        '!' =>      area!(405,38,6,34,0),
        '"' =>      area!(649,38,9,8,0),
        '%' =>      area!(604,45,12,20,7),
        '\'' =>     area!(644,38,3,8,0),
        '(' =>      area!(433,38,10,34,0),
        ')' =>      area!(445,38,10,34,0),
        '*' =>      area!(591,45,11,20,7),
        '+' =>      area!(509,45,14,20,7),
        ',' =>      area!(618,67,5,8,29),
        '-' =>      area!(525,45,14,20,7),
        '.' =>      area!(625,67,4,8,29),
        '/' =>      area!(481,38,12,34,0),
        '0' =>      area!(692,2,18,34,0),
        '1' =>      area!(513,2,15,34,0),
        '2' =>      area!(530,2,18,34,0),
        '3' =>      area!(550,2,18,34,0),
        '4' =>      area!(570,2,20,34,0),
        '5' =>      area!(592,2,18,34,0),
        '6' =>      area!(612,2,18,34,0),
        '7' =>      area!(632,2,18,34,0),
        '8' =>      area!(652,2,18,34,0),
        '9' =>      area!(672,2,18,34,0),
        ':' =>      area!(631,49,4,15,11),
        ';' =>      area!(637,49,5,15,11),
        '<' =>      area!(557,45,8,20,7),
        '=' =>      area!(541,45,14,20,7),
        '>' =>      area!(567,45,8,20,7),
        '?' =>      area!(413,38,18,34,0),
        'A' =>      area!(2,2,21,34,0),
        'B' =>      area!(25,2,18,34,0),
        'C' =>      area!(45,2,18,34,0),
        'D' =>      area!(65,2,18,34,0),
        'E' =>      area!(85,2,14,34,0),
        'F' =>      area!(101,2,14,34,0),
        'G' =>      area!(117,2,21,34,0),
        'H' =>      area!(140,2,18,34,0),
        'I' =>      area!(160,2,10,34,0),
        'J' =>      area!(172,2,15,34,0),
        'K' =>      area!(189,2,15,34,0),
        'L' =>      area!(206,2,10,34,0),
        'M' =>      area!(218,2,23,34,0),
        'N' =>      area!(243,2,18,34,0),
        'O' =>      area!(263,2,18,34,0),
        'P' =>      area!(283,2,16,34,0),
        'Q' =>      area!(301,2,18,34,0),
        'R' =>      area!(321,2,18,34,0),
        'S' =>      area!(341,2,18,34,0),
        'T' =>      area!(361,2,18,34,0),
        'U' =>      area!(381,2,20,34,0),
        'V' =>      area!(403,2,18,34,0),
        'W' =>      area!(423,2,28,34,0),
        'X' =>      area!(453,2,18,34,0),
        'Y' =>      area!(473,2,18,34,0),
        'Z' =>      area!(493,2,18,34,0),
        '[' =>      area!(457,38,10,34,0),
        '\\' =>     area!(495,38,12,34,0),
        ']' =>      area!(469,38,10,34,0),
        'a' =>      area!(2,53,16,19,15),
        'b' =>      area!(238,38,14,34,0),
        'c' =>      area!(20,53,12,19,15),
        'd' =>      area!(254,38,14,34,0),
        'e' =>      area!(34,53,16,19,15),
        'f' =>      area!(270,38,15,34,0),
        'g' =>      area!(323,53,14,30,15),
        'h' =>      area!(307,38,14,34,0),
        'i' =>      area!(232,47,4,25,9),
        'j' =>      area!(389,47,14,36,9),
        'k' =>      area!(287,38,12,34,0),
        'l' =>      area!(301,38,4,34,0),
        'm' =>      area!(52,53,21,19,15),
        'n' =>      area!(75,53,13,19,15),
        'o' =>      area!(90,53,14,19,15),
        'p' =>      area!(339,53,14,30,15),
        'q' =>      area!(355,53,16,30,15),
        'r' =>      area!(106,53,12,19,15),
        's' =>      area!(120,53,12,19,15),
        't' =>      area!(218,45,12,27,7),
        'u' =>      area!(134,53,13,19,15),
        'v' =>      area!(149,53,14,19,15),
        'w' =>      area!(165,53,20,19,15),
        'x' =>      area!(187,53,13,19,15),
        'y' =>      area!(373,53,14,30,15),
        'z' =>      area!(202,53,14,19,15),
        '|' =>      area!(577,45,12,20,7),
        _ => Default::default()
    }
}

impl FontDefinition for FontTwelven {
    fn get_texture(textures: &EngineTextures) -> TextureFrame {
        textures.font_twelven
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
    }
    fn get_glyph(character: char) -> GlyphArea {
        get_twelven_glyph_area(character)
    }
    const LINE_HEIGHT: f32 = 34.0;
    const LETTER_SPACING: f32 = 1.0;
    const WORD_SPACING: f32 = 8.0;
}

impl FontDefinition for FontMonoElf {
    fn get_texture(textures: &EngineTextures) -> TextureFrame {
        textures.font_mono_elf
    }

    fn get_glyph(character: char) -> GlyphArea {
        return match character {
            '0' =>          area!(1,37,5,11,0),
            '1' =>          area!(7,37,5,11,0),
            '2' =>          area!(13,37,5,11,0),
            '3' =>          area!(19,37,5,11,0),
            '4' =>          area!(25,37,5,11,0),
            '5' =>          area!(31,37,5,11,0),
            '6' =>          area!(37,37,5,11,0),
            '7' =>          area!(43,37,5,11,0),
            '8' =>          area!(49,37,5,11,0),
            '9' =>          area!(55,37,5,11,0),
            'a' | 'A' =>    area!(1,1,5,11,0),
            'b' | 'B' =>    area!(7,1,5,11,0),
            'c' | 'C' =>    area!(13,1,5,11,0),
            'd' | 'D' =>    area!(19,1,5,11,0),
            'e' | 'E' =>    area!(25,1,5,11,0),
            'f' | 'F' =>    area!(31,1,5,11,0),
            'g' | 'G' =>    area!(37,1,5,11,0),
            'h' | 'H' =>    area!(43,1,5,11,0),
            'i' | 'I' =>    area!(49,1,5,11,0),
            'j' | 'J' =>    area!(55,1,5,11,0),
            'k' | 'K' =>    area!(61,1,5,11,0),
            'l' | 'L' =>    area!(1,13,5,11,0),
            'm' | 'M' =>    area!(7,13,5,11,0),
            'n' | 'N' =>    area!(13,13,5,11,0),
            'o' | 'O' =>    area!(19,13,5,11,0),
            'p' | 'P' =>    area!(25,13,5,11,0),
            'q' | 'Q' =>    area!(31,13,5,11,0),
            'r' | 'R' =>    area!(37,13,5,11,0),
            's' | 'S' =>    area!(43,13,5,11,0),
            't' | 'T' =>    area!(1,25,5,11,0),
            'u' | 'U' =>    area!(7,25,5,11,0),
            'v' | 'V' =>    area!(13,25,5,11,0),
            'w' | 'W' =>    area!(19,25,5,11,0),
            'x' | 'X' =>    area!(25,25,5,11,0),
            'y' | 'Y' =>    area!(31,25,5,11,0),
            'z' | 'Z' =>    area!(37,25,5,11,0),
            ':' =>          area!(1,49,3,11,0),
            ';' =>          area!(4,49,3,11,0),
            '.' =>          area!(9,49,3,11,0),
            ',' =>          area!(13,49,3,11,0),
            '"' =>          area!(17,49,3,11,0),
            '\'' =>         area!(21,49,3,11,0),
            '/' =>          area!(25,49,3,11,0),
            '\\' =>         area!(29,49,3,11,0),
            '+' =>          area!(33,49,3,11,0),
            '-' =>          area!(37,49,3,11,0),
            '=' =>          area!(41,49,3,11,0),
            '%' =>          area!(45,49,3,11,0),
            '[' =>          area!(49,49,3,11,0),
            ']' =>          area!(53,49,3,11,0),
            '(' =>          area!(57,49,3,11,0),
            ')' =>          area!(61,49,3,11,0),
            '!' =>          area!(65,49,3,11,0),
            '?' =>          area!(69,49,3,11,0),
            '*' =>          area!(73,49,3,11,0),
            '<' =>          area!(81,49,3,11,0),
            '>' =>          area!(85,49,3,11,0),
            _ => Default::default()
        }
    }

    const LINE_HEIGHT: f32 = 11.0;
    const LETTER_SPACING: f32 = 1.0;
    const WORD_SPACING: f32 = 4.0;
}
