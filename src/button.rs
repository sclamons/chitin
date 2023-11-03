use sdl2::event::Event;
use crate::{state::{Settings, SimulatorComponents, SimulatorState}, simulator};

#[derive(Debug, Clone, Copy)]
pub enum ButtonID {
    PlayPause,
    StepForward,
    StepBackward,
    PlaybarBackground
}

pub fn process_click(
    event: &Event, 
    components: &mut SimulatorComponents,
    global_state: &mut SimulatorState, 
    settings: &Settings,
) {
    match event { 
        Event::MouseButtonDown{x, y, ..} => {
            for (i, bounding_box) in components.button_boxes.iter().enumerate() {
                if (bounding_box.left() <= *x) && (*x <= bounding_box.right())
                    && (bounding_box.bottom() >= *y) && (*y >= bounding_box.top()) {
                        global_state.pressed_button_idx = i;
                        break
                }
            }
        },
        Event::MouseButtonUp{x, y, ..} => {
            if global_state.pressed_button_idx >= components.button_boxes.len(){
                return;
            }
            let bounding_box = components.button_boxes[global_state.pressed_button_idx];
            if (bounding_box.left() <= *x) && (*x <= bounding_box.right())
                && (bounding_box.bottom() >= *y) && (*y >= bounding_box.top()) {
                    fire_mouse_event(
                        components.button_ids[global_state.pressed_button_idx], 
                        components, 
                        global_state, 
                        settings,
                        event
                    );
            }
            global_state.pressed_button_idx = components.button_boxes.len();
        },
        _ => panic!("Tried to process a non-click action as a click: {event:?}"),
    }
    
}

fn fire_mouse_event(
    button_id: ButtonID, 
    components: &mut SimulatorComponents,
    global_state: &mut SimulatorState, 
    settings: &Settings,
    event: &Event
) {
    match button_id {
        ButtonID::PlayPause => {
            // println!("Firing Playpause action.");
            global_state.is_playing = !global_state.is_playing;
            global_state.tick = global_state.is_playing;
        },
        ButtonID::StepForward => {
            let event = components.reaction_history[global_state.next_rxn_event];
            simulator::apply_reaction(
                &event, 
                global_state, 
                components, 
                settings,
                true);
            global_state.current_t = event.t;
            global_state.next_rxn_event += 1;
            if global_state.next_rxn_event >= components.reaction_history.len() {
                simulator::extend_reaction_history(components, global_state, settings);
            }
            // simulator::tick(global_state, components, settings);
        },
        ButtonID::StepBackward => {
            // println!("Firing StepBackward action.");
            if global_state.next_rxn_event > 0 {
                global_state.next_rxn_event -= 1;
                let event = components.reaction_history[global_state.next_rxn_event];
                simulator::apply_reaction(&event, global_state, components, settings, false);
                global_state.current_t = event.t;
            }
        },
        ButtonID::PlaybarBackground => {
            println!("Clicked in the playbar");
            if let Event::MouseButtonUp{x, ..} = *event {
                println!("Matched inside arm.");
                let x_position = x as f32 - components.positions[0].x;
                let x_frac = x_position / (settings.cell_size * settings.n_cols as u32) as f32;
                let new_rxn_idx = (x_frac * components.reaction_history.len() as f32) as usize;
                #[allow(clippy::comparison_chain)] if new_rxn_idx > global_state.next_rxn_event {
                    while global_state.next_rxn_event < new_rxn_idx {
                        let next_event = components.reaction_history[global_state.next_rxn_event];
                        simulator::apply_reaction(&next_event, global_state, components, settings, true);
                        global_state.current_t = next_event.t;
                        global_state.next_rxn_event += 1;
                        
                    }
                }
                else if new_rxn_idx < global_state.next_rxn_event {
                    while global_state.next_rxn_event >= new_rxn_idx && global_state.next_rxn_event > 0 {
                        let next_event = components.reaction_history[global_state.next_rxn_event - 1];
                        simulator::apply_reaction(&next_event, global_state, components, settings, false);
                        global_state.current_t = next_event.t;
                        global_state.next_rxn_event -= 1;
                    }
                }
            }
        }
    }
}