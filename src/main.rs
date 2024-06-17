mod renderer;
mod state;
mod input;
mod input_parsers;
mod simulator;
mod reactions;
mod textures;
mod button;

use sdl2::image::{self, InitFlag, LoadTexture};
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::surface::Surface;
use sdl2::ttf::{Sdl2TtfContext, Font};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::collections::HashMap;

use std::fs::File;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::input::{load_from_file, get_input_file};

// fn get_opengl_backend_idx() -> Option<u32>{ 
//     for (index, item) in sdl2::render::drivers().enumerate() {
//         if item.name == "opengl" {
//             return Some(index as u32);
//         }
//     }
//     None
// }

fn main() {
    let profiling = true;
    println!("Hello, world! Starting up...");

    // Initialize SDL2
    let sdl_context = sdl2::init().unwrap();
    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG).unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    match video_subsystem.gl_load_library_default() {
        Ok(()) => println!("Success loading OpenGL"),
        Err(_) => println!("Failed to load OpenGL default")
    };

    // Check backend
    println!("Backend info:");
    println!("Current backend is: {:?}", video_subsystem.current_video_driver());
    for renderer_info in sdl2::render::drivers() {
        println!("{renderer_info:?}");
    }
    
    // Set up fonts
    let text_context: Sdl2TtfContext = sdl2::ttf::init().unwrap();
    let default_font: Font = text_context.load_font("fonts/Swansea-q3pd.ttf",16).unwrap();

    // Load init file.
    let init_file =  get_input_file();
    let (mut sim_components, settings, mut global_state) = load_from_file(init_file);

    // Pre-render graphics and figure out how big the screen will need to be.
    let prerendered_surfaces = renderer::prerender_surfaces(
        &mut sim_components, 
        &settings, 
        &default_font
    );

    // Make a window
    let (window_width, window_height): (u32, u32) = renderer::calculate_window_size(&mut sim_components, &settings, &mut global_state, &prerendered_surfaces);
    let window = video_subsystem.window("Chitin", window_width, window_height)
        .opengl()
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas()
        // .index(get_opengl_backend_idx().unwrap())
        .accelerated()
        .build()
        .expect("could not make a canvas");
    println!("window.current_video_driver(): {:?}", canvas.window().subsystem().current_video_driver());
    // Prep textures
    let texture_creator = canvas.texture_creator();

    let mut prerendered_textures: HashMap<String, Texture> = prerendered_surfaces
        .into_iter()
        .map(|(name, surface)| (name, texture_creator.create_texture_from_surface(&surface).map_err(|e| e.to_string()).unwrap()))
        .collect();

    // Add button textures
    let button_sources = ["Play", "Next", "Back"];
    for (i, source) in button_sources.iter().enumerate() {
        prerendered_textures.insert(
            format!("{i}_button_up"),
            texture_creator.load_texture(Path::new(&format!("assets/Menu Buttons/Square Buttons/Square Buttons/{source} Square Button.png"))).unwrap()
        );
        prerendered_textures.insert(
            format!("{i}_button_down"),
            texture_creator.load_texture(Path::new(&format!("assets/Menu Buttons/Square Buttons/Colored Square Buttons/{source} col_Square Button.png"))).unwrap()
        );
    }
    let mut temp_surface = Surface::new(
        settings.cell_size * settings.n_cols as u32,
        renderer::PLAYBAR_BUTTON_HEIGHT,
        PixelFormatEnum::RGB24
    ).unwrap();
    temp_surface.fill_rect(
        Rect::new(
            0, 
            0,
            settings.cell_size * settings.n_cols as u32, 
            renderer::PLAYBAR_BUTTON_HEIGHT
        ),
        renderer::BACKGROUND_COLOR
    ).unwrap();
    temp_surface.fill_rect(
        Rect::new(
            0, 
            (renderer::PLAYBAR_BUTTON_HEIGHT as i32 - renderer::PLAYBAR_HEIGHT as i32) / 2,
            settings.cell_size * settings.n_cols as u32, 
            renderer::PLAYBAR_HEIGHT
        ),
        Color::RGB(50, 50, 50)
    ).unwrap();
    prerendered_textures.insert(
        format!("{}_button_up", button_sources.len()),
        temp_surface.as_texture(&texture_creator).unwrap()
    );
    prerendered_textures.insert(
        format!("{}_button_down", button_sources.len()),
        temp_surface.as_texture(&texture_creator).unwrap()
    );

    prerendered_textures.insert(
        "Pause_up".to_string(),
        texture_creator.load_texture(Path::new("assets/Menu Buttons/Square Buttons/Square Buttons/Pause Square Button.png")).unwrap()
    );
    prerendered_textures.insert(
        "Pause_down".to_string(),
        texture_creator.load_texture(Path::new("assets/Menu Buttons/Square Buttons/Colored Square Buttons/Pause col_Square Button.png")).unwrap()
    );

    let mut state_textures: HashMap<usize, Texture> = HashMap::new();
    
    for (state_id, class_id) in sim_components.state_colorclasses.iter() {
        let color = sim_components.colorclass_colors[*class_id];
        let mut temp_surface = Surface::new(
            settings.cell_size,
            settings.cell_size,
            PixelFormatEnum::RGB24
        ).unwrap();
        temp_surface.fill_rect(Rect::new(0, 0, settings.cell_size, settings.cell_size), color).unwrap();
        state_textures.insert(*state_id, temp_surface.as_texture(&texture_creator).unwrap());
    }

    // Initialized window settings
    canvas.set_draw_color(Color::RGB(200, 200, 220));
    canvas.clear();
    canvas.present();

    println!("Simulation starts on? {}", global_state.is_playing);

    // Set up the event queue
    simulator::initialize_queue(&sim_components, &mut global_state, &settings);

    // Set up event loop
    let mut event_pump = sdl_context.event_pump().unwrap();
    let one_tick = Duration::new(0, 1_000_000_000u32 / settings.fps as u32);
    let mut tick_start_time: Instant;
    let mut rxn_sim_start_time: Instant;
    let mut avg_rxn_sim_time = one_tick / 10; // Random assumption of starting simulation timing.
    let mut n_rxns_timed = 0;

    println!("Size of components.state_timestamps: {:?}", sim_components.state_timestamps.len());

    
    if profiling {
        flame::start("main");
    }
    
    /////////////////////
    // The event loop! //
    /////////////////////
    'running: loop {
        // Initialize tick
        tick_start_time = Instant::now();
        canvas.clear();

        // Check for inputs
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    break 'running;
                },
                Event::KeyDown{keycode: Some(Keycode::Space), ..} => {
                    global_state.is_playing = !global_state.is_playing;
                },
                Event::KeyDown{keycode: Some(Keycode::Left), ..} => {
                    global_state.run_direction_forward = false;
                    if !global_state.is_playing {
                        global_state.tick = true;
                    }
                },
                Event::KeyDown{keycode: Some(Keycode::Right), ..} => {
                    global_state.run_direction_forward = true;
                    if !global_state.is_playing {
                        global_state.tick = true;
                    }
                },
                Event::MouseButtonDown{..} | Event::MouseButtonUp{..} => {
                    button::process_click(&event, &mut sim_components, &mut global_state, &settings);
                }
                _ => {}
            }
        }
        // println!("Time processing inputs: {:?}", tick_start_time.elapsed());
        
        // Simulation logic
        if global_state.tick {
            simulator::tick(&mut global_state, &mut sim_components, &settings);
        }

        // Render results
        renderer::render(&mut canvas, &sim_components, &global_state, &settings, &default_font, &state_textures, &prerendered_textures);

        // Spend idle time building up simulation history, leaving a bit of buffer time.
        if one_tick > 2 * avg_rxn_sim_time && tick_start_time.elapsed() < one_tick - 2 * avg_rxn_sim_time {
            rxn_sim_start_time = Instant::now();
            simulator::extend_reaction_history(&mut sim_components, &mut global_state, &settings);
            avg_rxn_sim_time = (n_rxns_timed * avg_rxn_sim_time + rxn_sim_start_time.elapsed()) / (n_rxns_timed + 1);
            n_rxns_timed += 1;
        }
        else {
            // println!("Took too long ({:?})", tick_start_time.elapsed());
        }
    }
    
    if profiling {
        flame::end("main");
        flamescope::dump(&mut File::create("flamescope.json").unwrap()).unwrap();
    }
}