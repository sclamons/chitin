use std::fs::File;
use std::io::{BufRead, self};
use std::path::PathBuf;

use native_dialog::FileDialog;
use priq::PriorityQueue;

use crate::state::{Settings, SimulatorComponents, SimulatorState};
use crate::input_parsers::settings_input;

pub fn get_input_file() -> PathBuf {
    let path = FileDialog::new()
        .show_open_single_file()
        .unwrap();
    match path {
        Some(p)  => p,
        None => panic!("Failed to open an input file!")
    }
}

pub fn read_and_splice_settings_file(input_file: PathBuf) -> String {
    let file = match File::open(&input_file) {
        Err(why) => panic!("Couldn't open {input_file:?}: {why}", ),
        Ok(contents) => contents
    };
    let lines = io::BufReader::new(file).lines();
    let mut all_lines_string = String::new();
    for line in lines {
        let line = line.unwrap();
        if line.starts_with("!INCLUDE") {
            let mut included_path = input_file.clone();
            included_path.pop();
            included_path.push(&line.trim()[9..]);
            all_lines_string.push_str(&read_and_splice_settings_file(included_path));
        }
        else {
            all_lines_string.push_str(&line);
            all_lines_string.push('\n');
        }
    }
    println!("Reading out the following text from file {input_file:?}");
    println!("{}", &all_lines_string);
    println!("<End of text>");
    all_lines_string
}

pub fn load_from_file(input_file: PathBuf) -> (SimulatorComponents, Settings, SimulatorState) {
    let (components , settings) = 
        settings_input::settings(&read_and_splice_settings_file(input_file))
        .unwrap();
    // println!("File contents:\n{:?}", &fs::read_to_string(&input_file).unwrap());

    let global_state= SimulatorState {
        last_states: Vec::new(),
        rxn_queue: PriorityQueue::new(),
        speedup: 1.0,
        current_t: 0.0,
        next_rxn_event: 0,
        pressed_button_idx: components.button_boxes.len(),
        is_playing: false,
        run_direction_forward: true,
        tick: false
    };

    (components, settings, global_state)
}



#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::matches;
    use crate::state::SurfaceGeometry;

    use super::load_from_file;

    #[test]
    fn test_basic_settings_input() {
        let (sim_components, 
            settings, 
            _
        ) = load_from_file(PathBuf::from("./test_resources/manifests/basic_settings_manifest.txt"));

        assert_eq!(settings.margin, 60);
        assert_eq!(settings.cell_size, 14);
        assert_eq!(settings.fps, 30.0);
        assert!(settings.wrap);
        assert!(settings.debug);
        assert_eq!(settings.rng_seed, Some(12345));
        assert_eq!(settings.max_duration, 100.0);
        assert!(settings.display_text);
        assert!(matches!(settings.surface_geometry, SurfaceGeometry::Hex));
        let a_idx = *sim_components.state_ids.get("A").unwrap();
        let b_idx = *sim_components.state_ids.get("B").unwrap();
        assert_eq!(sim_components.current_states, vec![a_idx, a_idx, a_idx, a_idx, b_idx, a_idx, a_idx, a_idx, a_idx]);
    }

    #[test]
    fn test_default_settings_input() {
        let (sim_components, 
            settings, 
            _
        ) = load_from_file(PathBuf::from("test_resources/manifests/blank_settings_manifest.txt"));

        assert_eq!(settings.cell_size, 5);
        assert_eq!(settings.margin, 60);
        assert_eq!(settings.fps, 60.0);
        assert!(!settings.wrap);
        assert!(!settings.debug);
        assert_eq!(settings.rng_seed, None);
        assert_eq!(settings.max_duration, 1_000_000.0);
        assert!(!settings.display_text);
        assert!(matches!(settings.surface_geometry, SurfaceGeometry::Square));
        let a_idx = *sim_components.state_ids.get("A").unwrap();
        let b_idx = *sim_components.state_ids.get("B").unwrap();
        assert_eq!(sim_components.current_states, vec![a_idx, a_idx, a_idx, a_idx, b_idx, a_idx, a_idx, a_idx, a_idx]);
    }

    #[test]
    fn test_manifest_inclusion() {
        let (sim_components, 
            settings, 
            _
        ) = load_from_file(PathBuf::from("test_resources/manifests/manifest_with_include.txt"));

        println!("Loaded settings: {settings:?}");
        assert_eq!(settings.cell_size, 14);
        assert_eq!(settings.margin, 60);
        assert_eq!(settings.fps, 30.0);
        assert!(settings.wrap);
        let a_idx = *sim_components.state_ids.get("A").unwrap();
        let b_idx = *sim_components.state_ids.get("B").unwrap();
        assert_eq!(sim_components.current_states, vec![a_idx, a_idx, a_idx, a_idx, b_idx, a_idx, a_idx, a_idx, a_idx]);
    }
}