use rand::random;
// use sdl2::video::WindowContext;
use sdl2::{pixels::Color, rect::Rect};
use priq::PriorityQueue;

use std::collections::{HashMap, HashSet};

use crate::reactions::{Reaction, ReactionDescription, ReactionEvent};
use crate::button::ButtonID;
// use crate::textures::TextureAtlas;

#[derive(Debug)]
pub struct SimulatorComponents {
    pub sizes: Vec<Size>, // For graphics.
    pub positions: Vec<Position>, // For graphics.
    pub current_states: Vec<usize>, // Board state that is currently displayed.
    pub latest_states: Vec<usize>, // Highest-T simulated board state.
    pub state_timestamps: Vec<f32>, // The last time each position was changed.
    pub reaction_history: Vec<ReactionEvent>, // A list of all the events that have happened.
    pub all_reactions: Vec<Reaction>, // A list of reaction rules in the system.
    pub all_rxn_rates: Vec<f32>, 
    pub state_names: HashMap<usize, String>,
    pub state_colorclasses: HashMap<usize, usize>,
    pub colorclass_names: Vec<String>,
    pub colorclass_colors: Vec<Color>,
    pub state_ids: HashMap<String, usize>,
    pub button_boxes: Vec<Rect>,
    pub button_ids: Vec<ButtonID>,
    pub n_states_known: usize,
    pub n_colorclasses: usize
}

impl SimulatorComponents {
    pub fn new() -> Self {
        Self {
            sizes: Vec::new(),
            positions: Vec::new(),
            current_states: Vec::new(),
            latest_states: Vec::new(),
            state_timestamps: Vec::new(),
            reaction_history: Vec::new(),
            all_reactions: Vec::new(),
            all_rxn_rates: Vec::new(),
            state_names: HashMap::new(),
            state_colorclasses: HashMap::new(),
            colorclass_names: Vec::new(),
            colorclass_colors: Vec::new(),
            state_ids: HashMap::new(),
            button_boxes: Vec::new(),
            button_ids: Vec::new(),
            n_states_known: 0,
            n_colorclasses: 0
        }
    }

    pub fn new_random_color(&mut self) -> Color {
        Color::RGB(random::<u8>(), random::<u8>(), random::<u8>())
    }

    pub fn add_state(&mut self, name: &str, colorclass_id: Option<usize>) -> usize {
        let color_id = match colorclass_id {
            Some(c) => c,
            None => {
                self.colorclass_names.push(name.to_string());
                let new_color = self.new_random_color();
                self.colorclass_colors.push(new_color);
                self.n_colorclasses += 1;
                self.n_colorclasses - 1
            }
        };
        self.state_ids.insert(String::from(name), self.n_states_known);
        self.state_names.insert(self.n_states_known, String::from(name));
        self.state_colorclasses.insert(self.n_states_known, color_id);
        self.n_states_known += 1;
        self.n_states_known - 1
        // println!("State_names after insertion: {:?}", self.state_names);
    }

    pub fn add_color_class(&mut self, name: &str, color:&Color, states: &HashSet<String>) -> usize {
        self.colorclass_colors.push(*color);
        self.colorclass_names.push(name.to_owned());
        self.n_colorclasses += 1;
        for state in states {
            self.add_state(state, Some(self.n_colorclasses - 1));
            // println!("Inserting for state {state} ({:?})", self.state_ids.get(state));
            // self.state_colorclasses.insert(*self.state_ids.get(state).unwrap(), self.colorclass_names.len());
        }
        self.n_colorclasses - 1
    }

    pub fn add_transition_rule(&mut self, rule_description: &ReactionDescription) {
        for state in rule_description.all_states().into_iter() {
            if !self.state_ids.contains_key(state) {
                self.add_state(state, None);
            }
        }
        let new_rule = Reaction {
            r1_num: *(self.state_ids.get(&rule_description.r1).unwrap()),
            r2_num: match &rule_description.r2 {
                Some(name) => Some(*(self.state_ids.get(name).unwrap())),
                None => None
            },
            p1_num: *(self.state_ids.get(&rule_description.p1).unwrap()),
            p2_num: match &rule_description.p2 {
                Some(name) => Some(*(self.state_ids.get(name).unwrap())),
                None => None
            },
            rate: rule_description.rate
        };
        self.all_rxn_rates.push(new_rule.rate);
        self.all_reactions.push(new_rule);
    }

    // pub fn build_textures(&mut self, creator: &'a TextureCreator<WindowContext>, settings: &Settings) -> (){
    //     for (state, color) in self.state_colors.iter() {
    //         self.state_textures.insert(*state, TextureAtlas::build_cell_texture(color, creator, settings));
    //     }
    // }

    /// Sets a new board state as both current and last state, since future states may
    /// not be consistent with this one.
    pub fn set_board_state<'b, I>(&mut self, board_state: I, settings: &Settings)
    where I: Iterator<Item = &'b str>{
        let mut row = 0;
        let mut col = 0;
        for state_str in board_state {
            let state = *self.state_ids.get(state_str).unwrap();
            self.state_timestamps.push(0.0);
            self.current_states.push(state);
            self.latest_states.push(state);
            self.sizes.push(Size{width: settings.cell_size, height: settings.cell_size});
            self.positions.push(Position {
                x: (settings.margin + col * settings.cell_size) as f32, 
                y: (settings.margin + row * settings.cell_size) as f32
            });
            col += 1;
            if col as usize >= settings.n_cols {
                col = 0;
                row += 1;
            }
        }
        println!("Board state after setting: {:?}", self.current_states);
        println!("Dimensions (rows x cols) : ({:?}, {:?})", settings.n_rows, settings.n_cols);
    }

}

#[derive(Debug)]
pub struct SimulatorState {
    pub last_states: Vec<usize>,
    pub rxn_queue: PriorityQueue<f32, ReactionEvent>,
    pub speedup: f32,
    pub current_t: f32,
    pub next_rxn_event: usize,
    pub pressed_button_idx: usize,
    pub is_playing: bool,
    pub run_direction_forward: bool,
    pub tick: bool
}
#[derive(Debug)]

pub enum SurfaceGeometry{
    Square,
    Hex
}

#[derive(Debug)]
pub struct Settings {
    pub n_rows: usize,
    pub n_cols: usize,
    pub cell_size: u32,
    pub margin: u32,
    pub fps: f32,
    pub speedup_factor: f32,
    pub wrap: bool,
    pub debug: bool,
    pub rng_seed: Option<i32>,
    pub max_duration: f32,
    pub display_text: bool,
    pub surface_geometry: SurfaceGeometry
}

#[derive(Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}