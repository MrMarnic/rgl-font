use std::collections::HashMap;
use freetype::{Face, Library};
use freetype::face::LoadFlag;
use image::{ImageBuffer, Rgba};

fn calculate_value(v:f32) -> i32 {
    return (v as f32/64.0) as i32
}

fn load_char(face:&Face, char:char, font_height:f32, x_start:&mut i32, y_start: &mut i32, image:&mut ImageBuffer<Rgba<u8>, Vec<u8>>, padding_x:i32, padding_y:i32) -> Character{
    face.load_char(char as usize, LoadFlag::RENDER).unwrap();

    let glyph = face.glyph();
    let bitmap = glyph.bitmap();
    let w = bitmap.width();
    let h = bitmap.rows();
    let data = bitmap.buffer();

    let x_offset = glyph.bitmap_left();
    let x_advance = calculate_value(glyph.advance().x as f32);
    let y_offset = glyph.bitmap_top() - bitmap.rows();

    if *x_start+w > image.width() as i32{
        *y_start += calculate_value(face.size_metrics().unwrap().height as f32) + padding_y;
        *x_start = 10;
    }

    if char != ' ' {
        for y in 0..h {
            for x in 0..w {
                image.put_pixel((x + *x_start) as u32, (*y_start + y) as u32, Rgba([255,255,255,data[(x + y * w)as usize] as u8]));
            }
        }
    }

    let c = Character {
        id: char,
        x: *x_start,
        y: *y_start,
        width: w,
        height: h,
        x_offset: x_offset as f32,
        y_offset: y_offset as f32,
        x_advance: x_advance
    };

    *x_start += x_advance + padding_x;

    return c;
}

pub struct FontLoader {
    pub lib:Library
}

impl FontLoader {
    pub fn new() -> Self {
        FontLoader {
            lib: Library::init().unwrap()
        }
    }

    pub fn load_font(&self, name:&str,path:&str) -> BaseFont {
        let face = self.lib.new_face(path,0).unwrap();

        return BaseFont {
            name: name.to_string(),
            face
        }
    }
}

pub struct BaseFont {
    pub name:String,
    pub face:Face
}

impl BaseFont {
    pub fn scaled(&self,pixel_height:i32) -> ScaledFont {
        self.face.set_pixel_sizes(0, pixel_height as u32);

        let chars = r#"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!№;%:?*()_+-=.,/|"'@#$^&{}[]üäöß "#.chars().collect::<Vec<char>>();

        let mut im_w = 0;
        let mut im_h = 0;

        {
            let size = self.face.size_metrics().unwrap();

            let height = calculate_value(size.height as f32);
            let width = calculate_value(size.max_advance as f32);

            let w = 2000;

            let per_w = 2000/width;

            let y = chars.len() as i32 /per_w + 1;

            im_w = w;
            im_h = y * height + 1000;

        }

        let mut image: ImageBuffer<Rgba<u8>,Vec<u8>> = ImageBuffer::new(im_w as u32,im_h as u32);

        let mut yy = calculate_value(self.face.height() as f32) + 20;
        let mut xx = 10;
        let padding = 12;
        let padding_y = 20;

        let mut characters = HashMap::new();

        for c in chars {
            let character = load_char(&self.face,c,pixel_height as f32,&mut xx,&mut yy,&mut image,padding,padding_y);

            characters.insert(c,character);
        }

        return ScaledFont {
            pixel_height,
            chars: characters,
            image
        }
    }
}

pub struct Character {
    pub id:char,
    pub x:i32,
    pub y:i32,
    pub width:i32,
    pub height:i32,
    pub x_offset:f32,
    pub y_offset:f32,
    pub x_advance:i32
}

pub struct ScaledFont {
    pub pixel_height:i32,
    pub chars:HashMap<char,Character>,
    pub image:ImageBuffer<Rgba<u8>, Vec<u8>>
}

impl ScaledFont {
    pub fn get_width(&self, text:&String) -> f32 {
        let mut last_pos = 0.0;

        for (id,c) in text.chars().enumerate() {
            if !c.is_whitespace() {
                let char = &self.chars[&c];
                last_pos += char.x_advance as f32;
            } else {
                last_pos += 10.0;
            }
        }

        return last_pos;
    }

    pub fn get_width_chars(&self, chars:&Vec<char>) -> f32 {
        let mut last_pos = 0.0;

        for c in chars.iter() {
            let char = &self.chars[&c];
            last_pos += char.x_advance as f32;
        }

        return last_pos;
    }

    pub fn get_height(&self, text:&String) -> f32 {
        let mut last_pos = 0.0;
        let mut count = 0;

        for (id,c) in text.chars().enumerate() {
            if !c.is_whitespace() {
                let char = &self.chars[&c];
                last_pos += char.height as f32;
                count+=1;
            }
        }

        return last_pos / count as f32;
    }

    pub fn get_chars(&self, width:f32, str:&String) -> Option<usize> {
        let mut edit = str.clone();

        loop {
            if self.get_width(&edit) > width {
                if edit.len() > 1 {
                    edit.remove(edit.len()-1);
                } else {
                    return None
                }
            } else {
                break;
            }
        }

        return Some(edit.len() - 1);
    }

    pub fn get_chars_test(&self, width:f32, str:&Vec<char>) -> Option<usize> {

        if str.is_empty() {
            return None;
        }

        let mut edit = str.clone();

        loop {
            if self.get_width_chars(&edit) > width {
                if edit.len() > 1 {
                    edit.remove(edit.len()-1);
                } else {
                    return None
                }
            } else {
                break;
            }
        }

        return Some(edit.len() - 1);
    }
}
