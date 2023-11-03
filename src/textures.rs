// use sdl2::{render::{Texture, TextureCreator}, pixels::{Color, PixelFormatEnum}, video::WindowContext, surface::Surface, rect::Rect};

// use crate::state::Settings;

// // pub struct TextureAtlas<'a> {
// //     pub texture: Texture<'a>,
// //     pub width: u32,
// //     pub height:u32
// // }

// // impl<'a> Debug for TextureAtlas<'a> {
// //     fn fmt(&self, f: &mut Formatter <'_>) -> Result<(), Error> {
// //         write!(f, "Texture: [UNPRINTABLE]; dimensions: ({}, {})", self.width, self.height);
// //         Ok(())
// //     }
// // }

// // impl<'a> TextureAtlas<'a> {
// //     pub fn build_cell_texture(
// //         color: &Color, 
// //         creator: &'a TextureCreator<WindowContext>, 
// //         settings: &Settings
// //     ) -> TextureAtlas<'a> {

// //         let mut temp_surface = Surface::new(
// //             settings.cell_size as u32,
// //             settings.cell_size as u32,
// //             PixelFormatEnum::RGB24
// //         ).unwrap();
// //         temp_surface.fill_rect(Rect::new(0, 0, settings.cell_size, settings.cell_size as u32,), *color).unwrap();
        
// //         TextureAtlas {
// //             texture: temp_surface.as_texture(&creator).unwrap(),
// //             width: settings.cell_size as u32,
// //             height: settings.cell_size as u32
// //         }        
// //     }
// // }