use rand::random;

use crate::{state::{SimulatorState, SimulatorComponents, Settings}, reactions::{ReactionEvent, Reaction}};

/// Calculates the time in which a new reaction will fire, assuming that
/// its clock should start at the last-simulated event in the reaction history.
pub fn compute_next_t(
    components: &SimulatorComponents,
    rxn_idx: usize
) -> f32 {
    let t = match components.reaction_history.last() {
        Some(event) => event.t,
        None => 0.0
    };
    t + (1.0f32 / random::<f32>()).ln() / components.all_rxn_rates[rxn_idx]
}

/// Iterates through the indexes of the (square) neighbors of one position idx.
pub fn square_neighbors(idx: usize, width: usize, height: usize, wrap: bool) -> Vec<usize> {
    let mut neighbors: Vec<usize> = Vec::new();
    let x: i32 = (idx % (width)).try_into().unwrap();
    let y: i32 = (idx / width).try_into().unwrap();
    let directions: Vec<(i32,i32)> = vec![(-1, 0), (1, 0), (0, -1), (0, 1)];
    let mut new_x;
    let mut new_y: i32;
    for (dx, dy) in directions.iter() {
        new_x = x + dx;
        new_y = y + dy;
        if wrap {
            if new_x < 0 {
                new_x = (width as i32)-1;
            }
            else if new_x == width as i32 {
                new_x = 0;
            }
            if new_y < 0 {
                new_y = (height as i32)-1;
            }
            else if new_y == height as i32 {
                new_y = 0;
            }
            neighbors.push(new_x as usize + new_y as usize * width);
        }
        else if 0 <= new_x && new_x < width as i32 && 0 <= new_y && new_y < height as i32 {
            neighbors.push(new_x as usize + new_y as usize * width);
        }
    }
    neighbors
}

/// Keep popping reactions off the reaction queue until you get one that's valid, 
/// or there are none left.
/// 
/// Use this to figure out what the next reaction should be to push onto the 
/// reaction history.
pub fn pop_next_reaction(
    global_state: &mut SimulatorState,
    components: &mut SimulatorComponents, 
) -> Option<ReactionEvent> {
    let mut next_rxn_candidate: Option<ReactionEvent> = None;
    while next_rxn_candidate.is_none() {
        if global_state.rxn_queue.is_empty() {
            return None;
        }
        let rxn = global_state.rxn_queue.pop().unwrap().1;
        next_rxn_candidate = Some(rxn);
        // println!("{:?}", components.state_timestamps.len());
        // println!("{:?}", rxn.r1_loc);
        // println!("{:?}", rxn.r2_loc.unwrap());
        if rxn.t_issued < components.state_timestamps[rxn.r1_loc]
           || (rxn.r2_loc.is_some() && rxn.t_issued < components.state_timestamps[rxn.r2_loc.unwrap()]) {
            next_rxn_candidate = None;
        }
    }
    next_rxn_candidate
}

/// Update the current state given a reaction, assumed to be derived from 
/// state history (not from the reaction queue).
/// 
/// This should be used to *update the current state* after a tick. It does not
/// produce new reactions or add anything to the reaction history.
pub fn apply_reaction(
    next_event: &ReactionEvent, 
    global_state: &SimulatorState, 
    components: &mut SimulatorComponents, 
    settings: &Settings,
    forward: bool
) {
    let next_rxn: &Reaction = &components.all_reactions[next_event.rxn_idx];
    if forward {
        assert_eq!(components.current_states[next_event.r1_loc], next_rxn.r1_num);
        components.current_states[next_event.r1_loc] = next_rxn.p1_num;
        if next_event.r2_loc.is_some() {
            let r2_loc = next_event.r2_loc.unwrap();
            components.current_states[r2_loc] = next_rxn.p2_num.unwrap();
        }
    }
    else {
        assert_eq!(components.current_states[next_event.r1_loc], next_rxn.p1_num);
        components.current_states[next_event.r1_loc] = next_rxn.r1_num;
        if next_event.r2_loc.is_some() {
            assert_eq!(components.current_states[next_event.r2_loc.unwrap()], next_rxn.p2_num.unwrap());
            components.current_states[next_event.r2_loc.unwrap()] = next_rxn.r2_num.unwrap();
        }
    }
}

/// Adds one reaction to the surface state history, either when requested by tick 
/// or during idle time between frames. Always fires based on the last-simulated
/// event in the reaction history.
pub fn extend_reaction_history(components: &mut SimulatorComponents, global_state: &mut SimulatorState, settings: &Settings) {
    let maybe_next_event = pop_next_reaction(global_state, components); 
    if maybe_next_event.is_none() {
        return
    }
    let next_event: ReactionEvent = maybe_next_event.unwrap();
    let next_rxn = components.all_reactions[next_event.rxn_idx];
    components.reaction_history.push(next_event);

    // Apply changes from this new reaction to the last-computed state, including 
    // timestamp updates.
    components.latest_states[next_event.r1_loc] = next_rxn.p1_num;
    components.state_timestamps[next_event.r1_loc] = next_event.t;
    if next_event.r2_loc.is_some() {
        components.latest_states[next_event.r2_loc.unwrap()] = next_rxn.p2_num.unwrap();
        components.state_timestamps[next_event.r2_loc.unwrap()] = next_event.t;
    }
    
    // Check if (either of) the changed state(s) can react, and if so add those reactions
    // to the queue.
    check_for_new_reactions_at(next_event.r1_loc, components, global_state, settings, true);
    if next_event.r2_loc.is_some() {
        check_for_new_reactions_at(next_event.r2_loc.unwrap(), components, global_state, settings, true);
    }
}

/// Fills a channel with reaction events. Intended to be run in its own thread.
pub fn extend_reaction_history_threaded() {

}

/// Look for any reactions that can occur at this positon, and add them to the queue.
/// ONLY applies to the latest-simulated state -- earlier board states are 
/// already calculated. 
/// symmetric=true (what we usually want) means that the position can be either 
/// reactant; otherwise, only the first state can react (useful for setting up the 
/// initial reaction queue at the beginning of the simulation).
fn check_for_new_reactions_at(
    idx: usize,
    components: &SimulatorComponents,
    global_state: &mut SimulatorState,
    settings: &Settings,
    symmetric: bool
) {
    for rxn_idx in 0..components.all_reactions.len() {
        let rxn = &(components.all_reactions[rxn_idx]);
        if components.latest_states[idx] == rxn.r1_num {
            match rxn.r2_num {
                Some(r2) => {
                    let neighbors = square_neighbors(
                        idx, 
                        settings.n_cols, 
                        settings.n_rows, 
                        settings.wrap
                    );
                    for neighbor_idx in neighbors {
                        let neighbor_state = components.latest_states[neighbor_idx];
                        if neighbor_state == r2 {
                            let next_t = compute_next_t(components, rxn_idx);
                            let new_event = ReactionEvent{
                                r1_loc: idx,
                                r2_loc: Some(neighbor_idx),
                                rxn_idx: rxn_idx,
                                t: next_t,
                                t_issued: match components.reaction_history.last() {
                                    Some(event) => event.t,
                                    None => 0.0
                                }
                            };
                            // println!("(1) Adding ReactionEvent at t={next_t}: {new_event:?}");
                            global_state.rxn_queue.put(next_t, new_event);
                        }
                    }
                },
                None => {
                    let next_t = compute_next_t(components, rxn_idx);
                    let new_event = ReactionEvent{
                        r1_loc: idx,
                        r2_loc: None,
                        rxn_idx: rxn_idx,
                        t: next_t,
                        t_issued: match components.reaction_history.last() {
                            Some(event) => event.t,
                            None => 0.0
                        }
                    };
                    // println!("(2) Adding ReactionEvent at t={next_t}: {new_event:?}");
                    global_state.rxn_queue.put(next_t, new_event);
                }
            }
        } else if symmetric && rxn.r2_num.is_some() 
                    && components.latest_states[idx] == rxn.r2_num.unwrap() {
            let neighbors = square_neighbors(
                idx, 
                settings.n_cols, 
                settings.n_rows, 
                settings.wrap
            );
            for neighbor_idx in neighbors {
                let neighbor_state = components.latest_states[neighbor_idx];
                if neighbor_state == rxn.r1_num {
                    let next_t = compute_next_t(components, rxn_idx);
                    let new_event = ReactionEvent{
                        r1_loc: neighbor_idx,
                        r2_loc: Some(idx),
                        rxn_idx: rxn_idx,
                        t: next_t,
                        t_issued: match components.reaction_history.last() {
                            Some(event) => event.t,
                            None => 0.0
                        }
                    };
                    // println!("(3) Adding ReactionEvent at t={next_t}: {new_event:?}");
                    global_state.rxn_queue.put(next_t, new_event);
                }
            }
        }
    }
}

pub fn initialize_queue(
    components: &SimulatorComponents,
    global_state: &mut SimulatorState,
    settings: &Settings
) {
    for idx in 0..components.current_states.len() {
        check_for_new_reactions_at(idx, components, global_state, settings, false);
    }
}

// Maybe depricate this?
// pub fn add_reactions_after_event(
//     next_event: &ReactionEvent, 
//     global_state: &mut SimulatorState, 
//     components: &mut SimulatorComponents,
//     settings: &Settings
// ) {
//     let self_state = components.all_reactions[next_event.rxn_idx].p1_idx;
//     for rxn_idx in 0..components.all_reactions.len() {
//         let rxn = &(components.all_reactions[rxn_idx]);
//         if self_state == rxn.r1_idx {
//             let r2_idx = rxn.r2_idx;
//             match r2_idx {
//                 Some(r2) => {
//                     let neighbors = square_neighbors(
//                         next_event.r1_idx, 
//                         settings.n_cols, 
//                         settings.n_rows, 
//                         settings.wrap
//                     );
//                     for neighbor_idx in neighbors {
//                         let neighbor_state = components.latest_states[neighbor_idx];
//                         if neighbor_state == r2 {
//                             // Without this check, reactions with two of the same 
//                             // reactants will fire twice as often as intended.
//                             if (rxn.r1_idx == r2) && (neighbor_idx < next_event.r1_idx) {
//                                 continue;
//                             }
//                             let next_t = compute_next_t(components, rxn_idx);
//                             let new_event = ReactionEvent{
//                                 r1_idx: self_state,
//                                 r2_idx: Some(neighbor_state),
//                                 rxn_idx,
//                                 t: next_t,
//                                 t_issued: match components.reaction_history.last() {
//                                     Some(event) => event.t,
//                                     None => 0.0
//                                 }
//                             };
//                             global_state.rxn_queue.put(next_t, new_event);
//                         }
//                     }
//                 },
//                 None => {
//                     let next_t = compute_next_t(components, rxn_idx);
//                     let new_event = ReactionEvent{
//                         r1_idx: self_state,
//                         r2_idx: None,
//                         rxn_idx,
//                         t: next_t,
//                         t_issued: match components.reaction_history.last() {
//                             Some(event) => event.t,
//                             None => 0.0
//                         }
//                     };
//                     global_state.rxn_queue.put(next_t, new_event);
//                 }
//             }
//         }
//     }
// }

// This function updates the current surface state by one tick forward or backward, using
// pre-existing history if possible and creating more if necessary. 
pub fn tick(global_state: &mut SimulatorState, components: &mut SimulatorComponents, settings: &Settings) {
    if !(global_state.is_playing || global_state.tick) {
        return;
    }
    if global_state.run_direction_forward {
        global_state.current_t += settings.speedup_factor * 1.0 / settings.fps;
        if global_state.current_t > settings.max_duration {
            global_state.current_t = settings.max_duration;
            global_state.is_playing = false;
        }
        while global_state.next_rxn_event >= components.reaction_history.len() {
            extend_reaction_history(components, global_state, settings);
        }
        let mut next_event = components.reaction_history[global_state.next_rxn_event];
        while next_event.t <= global_state.current_t && next_event.t <= settings.max_duration {
            apply_reaction(&next_event, global_state, components, settings, true);
            global_state.next_rxn_event += 1;
            global_state.current_t = next_event.t;
            if global_state.next_rxn_event == components.reaction_history.len() {
                extend_reaction_history(components, global_state, settings);
            }
            next_event = components.reaction_history[global_state.next_rxn_event];
        }
    } else {
        global_state.current_t -= settings.speedup_factor * 1.0 / settings.fps;
        if global_state.current_t < 0.0 {
            global_state.current_t = 0.0;
            global_state.is_playing = false;
        }
        if global_state.next_rxn_event == 0 {
            return;
        }
        let mut next_event = components.reaction_history[global_state.next_rxn_event - 1];
        while next_event.t >= global_state.current_t {
            if global_state.next_rxn_event == 0 {
                break;
            }
            apply_reaction(&next_event, global_state, components, settings, false);
            global_state.next_rxn_event -= 1;
            global_state.current_t = next_event.t;
            if global_state.next_rxn_event > 0  {
                next_event = components.reaction_history[global_state.next_rxn_event - 1];
            }
        }
    }
    if !global_state.is_playing {
        global_state.tick = false;
    }
}


#[cfg(test)]
mod tests {
    use super::square_neighbors;

    fn coords_to_idx(x: usize, y: usize, width: usize) -> usize {
        x + y * width
    }
    
    // fn idx_to_coords(idx: usize, width: usize) -> (usize, usize) {
    //     let x: usize = idx % width;
    //     let y: usize = idx / width;
    //     return (x, y)
    // }

    #[macro_export]
    macro_rules! assert_vecs_equal {
        ($v1:expr, $v2:expr) => {
           assert_eq!($v1.len(), $v2.len());
           assert!($v2.iter().all(|item| $v1.contains(item)));
        }
    }

    #[test]
    fn test_square_interior() {
        let idx = coords_to_idx(1, 5, 3);
        assert_vecs_equal!(square_neighbors(idx, 3, 10, true), 
                          [(0, 5), (2, 5), (1, 4), (1, 6)].map(|(x, y)| coords_to_idx(x, y, 3)));
    }

    #[test]
    fn test_square_x_bounds() {
        // Wrap, left side
        let idx = coords_to_idx(0, 5, 3);
        assert_vecs_equal!(square_neighbors(idx, 3, 10, true),
                            [(2, 5), (1, 5), (0, 4), (0, 6)].map(|(x, y)| coords_to_idx(x, y, 3)));

        // Wrap, right side
        assert_vecs_equal!(square_neighbors(coords_to_idx(2, 5, 3), 3, 10, true),
                            [(1, 5), (0, 5), (2, 4), (2, 6)].map(|(x, y)| coords_to_idx(x, y, 3)));

        // No wrap, left side
        assert_vecs_equal!(square_neighbors(coords_to_idx(0, 5, 3), 3, 10, false),
                            [(1, 5), (0, 4), (0, 6)].map(|(x, y)| coords_to_idx(x, y, 3)));

        // No wrap, right side
        assert_vecs_equal!(square_neighbors(coords_to_idx(2, 5, 3), 3, 10, false),
                            [(1, 5), (2, 4), (2, 6)].map(|(x, y)| coords_to_idx(x, y, 3)));
    }

    #[test]
    fn test_square_y_bounds() {
        // Wrap, top side
        assert_vecs_equal!(square_neighbors(coords_to_idx(1, 0, 3), 3, 10, true),
                            [(0, 0), (2, 0), (1, 9), (1, 1)].map(|(x, y)| coords_to_idx(x, y, 3)));

        // Wrap, bottom side
        // println!("coords: {}", coords_to_idx(2, 9, 4));
        assert_vecs_equal!(square_neighbors(coords_to_idx(2, 9, 4), 4, 10, true),
                            [(1, 9), (3, 9), (2, 8), (2, 0)].map(|(x, y)| coords_to_idx(x, y, 4)));

        // No wrap, top side
        assert_vecs_equal!(square_neighbors(coords_to_idx(1, 0, 3), 3, 10, false),
                            [(0, 0), (2, 0), (1, 1)].map(|(x, y)| coords_to_idx(x, y, 3)));

        // No wrap, bottom side
        assert_vecs_equal!(square_neighbors(coords_to_idx(1, 9, 3), 3, 10, false),
                            [(0, 9), (2, 9), (1, 8)].map(|(x, y)| coords_to_idx(x, y, 3)));
    }
}