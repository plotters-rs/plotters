use rusttype::{Font, Scale, point};
use font_loader::system_fonts;
use std::collections::HashMap;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::cell::RefCell;

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

    pub fn draw<DrawFunc:FnMut(u32,u32,f32)>(&self, text: &str, (x,y):(u32,u32), mut draw:DrawFunc) -> bool {
        let scale = Scale::uniform(self.size as f32);
        if  self.font.borrow().is_none() {
            if let Some(font) = get_system_font(self.name) {
                self.font.replace(Some(font));
            } else {
                return false;
            }
        }

        if let Some(ref font) = *self.font.borrow() {
            font.layout(text, scale, point(x as f32, y as f32 + self.size as f32)).for_each(|g| {
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
