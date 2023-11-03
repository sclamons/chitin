use std::cmp::max;
use std::collections::HashMap;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{WindowCanvas, TextureQuery, Texture};
use sdl2::rect::Rect;

use itertools::izip;

use sdl2::surface::Surface;
use sdl2::ttf::Font;

use crate::button::ButtonID;
use crate::state::{SimulatorComponents, SimulatorState, Settings};

const MARGIN: u32 = 10;
const BUFFER: u32 = 5;
const BORDER_WIDTH: u32 = 2;
const BUTTON_WIDTH: u32 = 25;
const BUTTON_HEIGHT: u32 = 25;
pub const PLAYBAR_BUTTON_HEIGHT: u32 = 12;
pub const PLAYBAR_HEIGHT: u32 = 2;

pub const BACKGROUND_COLOR: Color = Color::RGB(200, 200, 220);

fn playbar_y(sim_components: &SimulatorComponents, settings: &Settings) -> i32 {
    ((settings.cell_size as i32 * settings.n_rows as i32 + sim_components.positions[0].y as i32) 
    + sim_components.button_boxes.iter().map(|rect| {rect.top()}).max().unwrap()) / 2
}

pub fn prerender_surfaces<'a>(
    sim_components: &mut SimulatorComponents,
    _settings: &Settings,
    default_font: &Font
) -> HashMap<String, Surface<'a>> {
    let mut prerendered_surfaces: HashMap<String, Surface> = HashMap::new();
    
    prerendered_surfaces.insert(
        "legend".to_string(), 
        generate_legend_surface(
            sim_components,
            default_font
        )
    );

    // let standard_button_rect = Rect::new(0, 0, BUTTON_WIDTH, BUTTON_HEIGHT);
    // let mut button_surface = Surface::new(BUTTON_WIDTH, BUTTON_HEIGHT, PixelFormatEnum::RGB24).unwrap();
    // button_surface.fill_rect(standard_button_rect, Color::RGB(0, 255, 0));
    // prerendered_surfaces.insert(
    //     "button".to_string(),
    //     button_surface
    // );
    // let mut button_down_surface = Surface::new(BUTTON_WIDTH, BUTTON_HEIGHT, PixelFormatEnum::RGB24).unwrap();
    // button_down_surface.fill_rect(standard_button_rect, Color::RGB(255, 0, 0));
    // prerendered_surfaces.insert("down_button".to_string(), button_down_surface);

    prerendered_surfaces
}

pub fn generate_legend_surface<'a>(
    components: &SimulatorComponents, 
    legend_font: &Font
) -> Surface<'a> {
    let mut legend_height = 2 * MARGIN;
    let mut legend_width = 0;
    for (_index, class_name) in components.colorclass_names.iter().enumerate() {
        // let font_surface = legend_font
        //     .render(class_name)
        //     .blended(Color::RGBA(0, 0, 0, 255))
        //     .unwrap();
        let (mut text_width, text_height) = legend_font.size_of(class_name).unwrap();
        legend_height += text_height + BUFFER;
        text_width += 2 * MARGIN + text_height + BUFFER;
        if text_width > legend_width {
            legend_width = text_width;
        }
    }
    legend_height -= BUFFER;

    let mut legend_surface = Surface::new(legend_width, legend_height, PixelFormatEnum::RGB24).unwrap();
    legend_surface.fill_rect(Rect::new(0, 0, legend_width, legend_height), Color::RGB(255, 255, 255)).ok();
    let x = BUFFER;
    let mut y = BUFFER;

    for i in 0..components.colorclass_names.len() {
        let state_name = components.colorclass_names[i].clone();
        let color = components.colorclass_colors[i];
        let text_props = legend_font.size_of(&state_name).unwrap();
        let font_height = legend_font.height() as u32;
        legend_surface.fill_rect(
            Rect::new((x-1) as i32, (y-1) as i32, font_height+2, font_height+2), 
            Color::RGB(0,0,0)
        ).ok();
        legend_surface.fill_rect(
            Rect::new(x as i32, y as i32, font_height, font_height), 
            color
        ).unwrap();
        let font_surface = legend_font
            .render(&state_name)
            .blended(Color::RGBA(0, 0, 0, 255))
            .unwrap();
        font_surface.blit(
            None, 
            &mut legend_surface, 
            Rect::new(
                (x + font_surface.height() + BUFFER) as i32, 
                y as i32, 
                font_surface.width(), 
                font_surface.height()
            )
        ).unwrap();
        y += text_props.1 + BUFFER;
    }
    legend_surface
    // legend_texts[0].clone()
}


pub fn calculate_window_size(
    sim_components: &mut SimulatorComponents, 
    settings: &Settings,
    state: &mut SimulatorState,
    prerendered_surfaces: &HashMap<String, Surface>) 
    -> (u32, u32) {
    let surface_width: u32 = settings.n_cols as u32 * settings.cell_size;
    let surface_height: u32 = settings.n_rows as u32 * settings.cell_size;

    // Also place buttons here, which is a bit of a hack.
    sim_components.button_boxes.push(Rect::new(
        (settings.margin + surface_width/2) as i32, 
        (settings.margin * 2 + surface_height) as i32,
        BUTTON_WIDTH,
        BUTTON_HEIGHT
    ));
    sim_components.button_ids.push(ButtonID::PlayPause);

    sim_components.button_boxes.push(Rect::new(
        (settings.margin + surface_width/2 + (BUFFER + BUTTON_WIDTH)) as i32, 
        (settings.margin * 2 + surface_height) as i32,
        BUTTON_WIDTH,
        BUTTON_HEIGHT
    ));
    sim_components.button_ids.push(ButtonID::StepForward);

    sim_components.button_boxes.push(Rect::new(
        (settings.margin + surface_width/2 - (BUFFER + BUTTON_WIDTH)) as i32, 
        (settings.margin * 2 + surface_height) as i32,
        BUTTON_WIDTH,
        BUTTON_HEIGHT
    ));
    sim_components.button_ids.push(ButtonID::StepBackward);

    // canvas.set_draw_color(Color::RGB(50, 50,50));
    sim_components.button_boxes.push(
        Rect::new(
            sim_components.positions[0].x as i32,
            playbar_y(sim_components, settings),
            settings.cell_size * settings.n_cols as u32,
            PLAYBAR_BUTTON_HEIGHT
        )
    );
    sim_components.button_ids.push(ButtonID::PlaybarBackground);

    state.pressed_button_idx = sim_components.button_boxes.len();

    (
        surface_width + 3 * settings.margin + prerendered_surfaces["legend"].width(),
        max(surface_height, prerendered_surfaces["legend"].height()) + 3 * settings.margin + BUTTON_HEIGHT
    )
}

pub fn render(
    canvas: &mut WindowCanvas,
    components: &SimulatorComponents,
    state: &SimulatorState,
    settings: &Settings,
    font: &Font,
    state_textures: &HashMap<usize, Texture>,
    prerendered_textures: &HashMap<String, Texture>) 
    {
    // Setup
    canvas.set_draw_color(BACKGROUND_COLOR);
    canvas.clear();
    let texture_creator = canvas.texture_creator();

    // Time
    canvas.set_draw_color(Color::RGB(255,0,0));
    let surface = font
        .render(&format!("T = {:0.2}", state.current_t))
        .blended(Color::RGBA(0, 0, 0, 255))
        .map_err(|e| e.to_string()).unwrap();
    let time_text_texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string()).unwrap();
    let TextureQuery { width, height, .. } = time_text_texture.query();
    canvas.copy(&time_text_texture, None, 
        Rect::new(
            components.positions[0].x as i32, 
            components.positions[0].y as i32 - 20, 
            width, 
            height
        )).unwrap();

    // Legend
    let legend_texture = &prerendered_textures["legend"];
    let legend_x = 2 * BUFFER + settings.margin + settings.n_cols as u32 * settings.cell_size;
    let legend_y = settings.margin;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.draw_rect(
        Rect::new(
            (legend_x - BORDER_WIDTH) as i32, 
            (legend_y - BORDER_WIDTH) as i32, 
        legend_texture.query().width + 2 * BORDER_WIDTH,
        legend_texture.query().height + 2 * BORDER_WIDTH
        )
    ).unwrap_or_else(|err| println!("Failed to draw legend: {err}"));

    canvas.copy(
        legend_texture, 
        None, 
        Rect::new(
            legend_x as i32,
            legend_y as i32,
            legend_texture.query().width, 
            legend_texture.query().height
        )
    ).unwrap();

    // Buttons
    let mut button_texture: &Texture;
    for (i, button_box) in components.button_boxes.iter().enumerate() {
        if i == state.pressed_button_idx {
            if i==0 && state.is_playing {
                button_texture = &prerendered_textures["Pause_down"];
            }
            else {
                button_texture = &prerendered_textures[&format!("{i}_button_down")];
            }
        }
        else if i==0 && state.is_playing {
            button_texture = &prerendered_textures["Pause_up"];
        }
        else {
            button_texture = &prerendered_textures[&format!("{i}_button_up")];
        }
        canvas.copy(button_texture, None, *button_box).ok();
    }

    // Playbar vertical bar
    canvas.set_draw_color(Color::RGB(0, 0,0));
    canvas.draw_rect(
        Rect::new(
            (components.positions[0].x + (state.next_rxn_event as f32 / (max(2, components.reaction_history.len()) - 1) as f32) * (settings.cell_size as f32 * settings.n_cols as f32)) as i32 - 1,
            playbar_y(components, settings),
            3,
            PLAYBAR_BUTTON_HEIGHT
        )
    ).unwrap();

    // Surface
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.draw_rect(
        Rect::new(
            (components.positions[0].x - BORDER_WIDTH as f32) as i32, 
            (components.positions[0].y - BORDER_WIDTH as f32) as i32, 
        (settings.cell_size * settings.n_cols as u32) + 2 * BORDER_WIDTH,
        (settings.cell_size * settings.n_rows as u32) + 2 * BORDER_WIDTH
        )
    ).unwrap();
    for (state, position, size) in izip!(components.current_states.iter(), components.positions.iter(), components.sizes.iter()) {
        let texture = state_textures.get(state).unwrap();
        canvas.copy(texture, None, Rect::new(position.x as i32, position.y as i32, size.width, size.height)).ok();
    }

    canvas.present();
}