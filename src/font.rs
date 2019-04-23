use rusttype::{Font, Scale, point};
use font_loader::system_fonts;
use std::collections::HashMap;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::cell::RefCell;
use std::i32;

lazy_static! {
    static ref FONT_DATA_CACHE : Mutex<HashMap<String, Vec<u8>>> = {
        Mutex::new(HashMap::new())
    };
}

fn load_font_data(face:&str) -> Option<&'static [u8]> {
    match FONT_DATA_CACHE.lock().map(|mut cache| {
        let query = system_fonts::FontPropertyBuilder::new().family(face).build();
        if let Some((data, _)) = system_fonts::get(&query) {
            cache.insert(face.to_string(), data);
            return Some(unsafe{std::mem::transmute::<_, &'static [u8]>(&cache.get(face).unwrap()[..])});
        }
        return None;
    }) {
        Ok(what) => what,
        Err(_) => None,
    }
}

fn get_system_font(face:&str) -> Option<Font<'static>> {
    return load_font_data(face).map(|x| {
        return Font::from_bytes(x).unwrap();
    });
}

pub struct FontDesc<'a> {
    size: f64,
    name: &'a str,
    font: RefCell<Option<Font<'a>>>,
}

impl <'a> FontDesc<'a> {
    pub fn new(typeface:&'a str, size: f64) -> Self {
        return Self {
            size,
            name: typeface,
            font: RefCell::new(None)
        };
    }

    pub fn get_name(&self) -> &'a str {
        return self.name;
    }

    pub fn get_size(&self) -> f64 {
        return self.size;
    }

    pub fn box_size(&self, text: &str) -> (u32, u32) {
        let scale = Scale::uniform(self.size as f32);
        
        if  self.font.borrow().is_none() {
            if let Some(font) = get_system_font(self.name) {
                self.font.replace(Some(font));
            } else {
                return (0,0);
            }
        }

        if let Some(ref font) = *self.font.borrow() {
            let (mut min_x, mut min_y) = (i32::MAX, i32::MAX);
            let (mut max_x, mut max_y) = (0, 0);

            font.layout(text, scale, point(0 as f32, 0 as f32)).for_each(|g| {
                if let Some(rect) = g.pixel_bounding_box() {
                    min_x = min_x.min(rect.min.x);
                    min_y = min_y.min(rect.min.y);
                    max_x = max_x.max(rect.max.x);
                    max_y = max_y.max(rect.max.y);
                }
            });

            if min_x == i32::MAX || min_y == i32::MAX {
                return (0,0);
            }

            return ((max_x - min_x) as u32,
                    (0 - min_y) as u32);
        }
        return (0,0);
    }

    pub fn draw<DrawFunc:FnMut(u32,u32,f32)>(&self, text: &str, (x,y):(u32,u32), mut draw:DrawFunc) -> bool {
        let (_, h) = self.box_size(text);

        let scale = Scale::uniform(self.size as f32);
        if  self.font.borrow().is_none() {
            if let Some(font) = get_system_font(self.name) {
                self.font.replace(Some(font));
            } else {
                return false;
            }
        }

        if let Some(ref font) = *self.font.borrow() {
            font.layout(text, scale, point(x as f32, y as f32 + h as f32)).for_each(|g| {
                if let Some(rect) = g.pixel_bounding_box() {
                    let x0 = rect.min.x;
                    let y0 = rect.min.y;
                    g.draw(|x,y,v| {
                        if x as i32 + x0 >= 0 && y as i32 + y0 >= 0 {
                            draw((x as i32 + x0) as u32, (y as i32 + y0) as u32, v);
                        }
                    });
                }
            });
        }

        return true;
    }
} 
