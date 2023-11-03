use std::collections::{HashMap, HashSet};

use sdl2::pixels::Color;

use crate::state::{SimulatorComponents, Settings, SurfaceGeometry};
use crate::reactions::ReactionDescription;

#[derive(Debug)]
enum InputBlock {
    None,
    VariableLine(String, String), // Variable, Value
    InitStateBlock(Vec<String>, u32, u32), // States, number of rows, numbr of columns
    SingleTransitionRule(ReactionDescription),
    TransitionRuleBlock(Vec<ReactionDescription>),
    SingleColormap((String, (Color, HashSet<String>))),
    ColormapBlock(HashMap<String, (Color, HashSet<String>)>), // Maps color class -> (color, set(states))
}

peg::parser!{
    pub grammar settings_input() for str {
        pub rule settings() -> (SimulatorComponents, Settings)
         = all_lines:((line() / init_state_block() / transition_rule_block() / colormap_block() / comment() / blank()) ** ['\n']) {
            //////////////////////// 
            // SETTINGS VARIABLES //
            ////////////////////////
            let variables: HashMap<String, String> = all_lines
                .iter()
                .filter_map(|block| match block {
                    InputBlock::VariableLine(var, val) => Some((var.to_string(), val.to_string())),
                    _ => None
                })
                .collect();
                         
            let mut settings = Settings {
                n_rows: 0,
                n_cols: 0,
                cell_size: variables.get("pixels_per_node").map_or(5, |s| s.parse::<u32>().unwrap()),
                margin: 60,
                fps: variables.get("fps").map_or(60.0, |s| s.parse::<f32>().unwrap()),
                speedup_factor: variables.get("speedup_factor").map_or(1.0, |s| s.parse::<f32>().unwrap()),
                wrap: variables.get("wrap_grid").map_or(false, |s| s.parse::<bool>().unwrap()),
                debug: variables.get("debug").map_or(false, |s| match s.to_lowercase().as_str() {
                    "true" | "on" | "yes" | "1" => true,
                    "false" | "off" | "no" | "0" => false,
                    _ => false
                }),
                rng_seed: variables.get("rng_seed").map(|s| s.parse::<i32>().unwrap()),
                max_duration: variables.get("max_duration").map_or(1_000_000.0, |s| s.parse::<f32>().unwrap()),
                display_text: match variables.get("display_text") {
                        Some(s) => Some(&s[..]), 
                        None => match variables.get("node_text") {
                            Some(s) => Some(&s[..]), 
                            None => Some("false")
                        }
                    }.map_or(false, |s| match s.to_lowercase().as_str() {
                    "true" | "yes" | "text" => true,
                    "false" | "no" | "color" => false,
                    _ => false
                }),
                surface_geometry: variables.get("surface_geometry").map_or(SurfaceGeometry::Square, |s| match s.to_lowercase().as_str() {
                    "hex" | "hexagonal" | "hexagons" | "honeycomb" => SurfaceGeometry::Hex,
                    "square" | "box" | "grid" => SurfaceGeometry::Square,
                    _ => SurfaceGeometry::Square
                })
            };

            //////////////
            // COLORMAP //
            //////////////
            
            let mut components = SimulatorComponents::new();
            let mut state_to_class_id: HashMap<String, usize> = HashMap::new();
            
            let colormap_block_candidates: Vec<&HashMap<String, (Color, HashSet<String>)>> = all_lines
                .iter()
                .filter_map(|block| match block {
                    InputBlock::ColormapBlock(colormap) => Some(colormap),
                    _ => None
                })
                .collect();

            for colormap_block in colormap_block_candidates {
                for (class_name, (color, states)) in colormap_block {
                    // components.color_classes.insert(class_name.clone(), *color);
                    let class_id = components.add_color_class(class_name, color, states);
                    for state in states {
                        state_to_class_id.insert((*state).clone(), class_id);
                    }
                }
            }

            ////////////////
            // INIT STATE //
            ////////////////
            
            let init_state_block_candidates: Vec<(&Vec<String>, u32, u32)> = all_lines
                .iter()
                .filter_map(|block| match block {
                    InputBlock::InitStateBlock(state_vec, n_rows, n_cols) => Some((state_vec, *n_rows, *n_cols)),
                    _ => None
                })
                .collect();
            if init_state_block_candidates.len() != 1 {
                if init_state_block_candidates.is_empty() {
                    panic!("Couldn't find an initial state in manifest! Input blocks are: {all_lines:?}");
                }
                panic!("Too many or too few initial states in manifest!");
            }

            let (init_state_string_parts, n_rows, n_cols) = init_state_block_candidates[0];
            let all_states: HashSet<&str>  = init_state_string_parts.iter().map(|s| &s[..]).collect();

            for state in all_states {
                // println!("Adding state {state} to components.");
                components.add_state(state, state_to_class_id.get(state).copied());
            }
            
            settings.n_rows = n_rows as usize;
            settings.n_cols = n_cols as usize;
            if components.current_states.len() % settings.n_rows != 0 {
                panic!("This initial state isn't square: {init_state_string_parts:?}");
            }
            
            components.set_board_state(init_state_string_parts.iter().map(|s| &s[..]), &settings);

            //////////////////////
            // TRANSITION RULES //
            //////////////////////
            
            let transition_rule_blocks: Vec<(&Vec<ReactionDescription>)> = all_lines
                .iter()
                .filter_map(|block| match block {
                    InputBlock::TransitionRuleBlock(rules) => Some(rules),
                    _ => None
                })
                .collect();

            for rule_block in transition_rule_blocks {
                for rule in rule_block {
                    components.add_transition_rule(rule);
                }
            }

            ////////////
            // RETURN //
            ////////////
        
            (components, settings)
         }

        rule line() -> InputBlock
         = setting:variable() [' ']* "=" [' ']* val:value() 
         {
            InputBlock::VariableLine(setting, val)
        }

        rule variable() -> String
         = v:$("pixels_per_node" / "fps" / "wrap_grid" / "speedup_factor" / "debug" 
                / "rng_seed" / "max_duration" / "display_text" / "node_display" / "surface_geometry") 
              {String::from(v)}

        rule value() -> String
         = v:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '_']+) {String::from(v)}

         ////// START HERE ///////
         // Problem 1) This rule doesn't allow states starting with digits (i.e., 1Ax)
         // Problem 2) If you delete the first bit that just matches letters, then 
         //             this rule *does* match to "!END_INIT_STATE" for some reason, which breaks blocks.
        rule state() -> String 
        = v:$(['a'..='z' | 'A'..='Z' | '0'..='9']+) {String::from(v)}

        rule comment() -> InputBlock
         = "#" [^'\n']* {InputBlock::None}

        rule blank() -> InputBlock
         = blank:$(" "*) {InputBlock::None}

        rule init_state_block() -> InputBlock
         = "!START_INIT_STATE\n" state:$(state() ** ([',' | '\n' | ' ' | '\t']*)) "\n!END_INIT_STATE" 
            {
                let state_bits: Vec<String> = state.split_whitespace().map(|s| s.to_string()).collect();
                let n_rows = 1 + state.chars().filter(|s| *s == '\n').count();
                let n_cols = (state_bits.len() / n_rows);
                InputBlock::InitStateBlock(state_bits, n_rows as u32, n_cols as u32)
            }

        rule transition_rule_block() -> InputBlock
         = "!START_TRANSITION_RULES\n" transition_rules:(((comment() / unimolecular_rule() / bimolecular_rule() / ws())) ** ['\n']) "!END_TRANSITION_RULES"
            {
                InputBlock::TransitionRuleBlock(
                    transition_rules
                    .into_iter()
                    .filter_map(|line| match line 
                        {
                            InputBlock::SingleTransitionRule(r) => Some(r),
                            _ => None
                        }
                    )
                    .collect())
            }
        
        rule unimolecular_rule() -> InputBlock
         = r:(rate_first_unimolecular_rule() / rate_last_unimolecular_rule()) {r}
        
        rule rate_first_unimolecular_rule() -> InputBlock
         = ws() r1:$(state()) ws() "->" ws() p1:$(state()) ws() rate:(rate()) ws()
         {
            InputBlock::SingleTransitionRule(
                ReactionDescription {
                    r1: r1.to_string(), 
                    r2: None, 
                    p1: p1.to_string(), 
                    p2: None, 
                    rate
                }
            )
         }

        rule rate_last_unimolecular_rule() -> InputBlock
         = ws() rate:(rate()) ws() r1:$(state()) ws() "->" ws() p1:$(state()) ws()
         {
            InputBlock::SingleTransitionRule(
                ReactionDescription {
                    r1: r1.to_string(), 
                    r2: None, 
                    p1: p1.to_string(), 
                    p2: None, 
                    rate
                }
            )
         }

        rule bimolecular_rule() -> InputBlock 
         = r:(rate_first_bimolecular_rule() / rate_last_bimolecular_rule()) {r}

        rule rate_first_bimolecular_rule() -> InputBlock
         = ws() r1:$(state()) ws() "+" ws() r2:$(state()) ws() "->" ws() p1:$(state()) ws() "+" ws() p2:$(state()) ws() rate:(rate()) ws()
         {
            InputBlock::SingleTransitionRule(
                ReactionDescription {
                    r1: r1.to_string(), 
                    r2: Some(r2.to_string()), 
                    p1: p1.to_string(), 
                    p2: Some(p2.to_string()),
                    rate
                }
            )
         }

         rule rate_last_bimolecular_rule() -> InputBlock
         = ws() rate:(rate()) ws() r1:$(state()) ws() "+" ws() r2:$(state()) ws() "->" ws() p1:$(state()) ws() "+" ws() p2:$(state()) ws()
         {
            InputBlock::SingleTransitionRule(
                ReactionDescription {
                    r1: r1.to_string(),
                    r2: Some(r2.to_string()),
                    p1: p1.to_string(),
                    p2: Some(p2.to_string()),
                    rate
                }
            )
         }

        rule rate() -> f32
         = "(" rate_num:$(['0'..='9']* ("." ['0'..='9']+)?) ")" {rate_num.parse::<f32>().unwrap()}

        rule colormap_block() -> InputBlock
         = "!START_COLORMAP\n" colors:((comment() / color_class() / ws()) ** ['\n']) "!END_COLORMAP"
         {
            let colormap: HashMap<String, (Color, HashSet<String>)> = colors
                .into_iter()
                .filter_map(|line| match line {InputBlock::SingleColormap(cm) => Some(cm), _ => None})
                .collect();
            InputBlock::ColormapBlock(colormap)
         }

        rule color_class() -> InputBlock
         = class:(single_color_def() / color_class_def())
        {
            InputBlock::SingleColormap(class)
        }

        rule single_color_def() -> (String, (Color, HashSet<String>))
         = ws() s:state() ws() ":" ws() c:color_tuple() ws()
         {
            let mut class_colors: HashSet<String> = HashSet::new();
            class_colors.insert(s.clone());
            (s, (c, class_colors))
         }

        rule color_class_def() -> (String, (Color, HashSet<String>))
         = "{" class_name:([^'}']+) "}" ws() states:((state()) ** ([' ']* [','] [' ']*)) ws() ":" ws() c:color_tuple() ws()
         {
            let class_colors: HashSet<String> = states.into_iter().collect();
            (class_name.into_iter().collect::<String>(), (c, class_colors))
         }

        rule color_tuple() -> Color
         = "(" ws() r:(rgb_num()) ws() "," ws() g:(rgb_num()) ws() "," ws() b:(rgb_num()) ws() ")"
        {
            Color::RGB(r, g, b)
        }

        rule rgb_num() -> u8
         = n:((['0'..='9'])*<1,3>) {
            let s = n.into_iter().collect::<String>();
            match s.parse::<u8>() {
                Ok(i) => i,
                Err(error) => panic!("Failed to turn this number into a u8: {s:?}")
            }
        }

        rule ws() -> InputBlock = [' ']* {InputBlock::None}
    }
}
