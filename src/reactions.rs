// 
// Indexes ending *_num are for indexing into generic descriptors,
// e.g. reaction rules or species identities.
//
// Indexes ending in *_loc are for indexing into instances linked to locations,
// e.g. the location of a reaction event.
//
// Indexes ending in ??? are for indexing into individual instances
// NOT linked to locations, e.g. the index of a specific reaction event
// in a reaction queue.
//

// A reaction rule, with reactants, products, and rate.
#[derive(Debug, Clone, Copy)]
pub struct Reaction {
    pub r1_num: usize,          //
    pub r2_num: Option<usize>,
    pub p1_num: usize,
    pub p2_num: Option<usize>,
    pub rate: f32
}

//Stores plaintext description of the reaction.
#[derive(Debug)]
pub struct ReactionDescription {
    pub r1: String,
    pub r2: Option<String>,
    pub p1: String,
    pub p2: Option<String>,
    pub rate: f32
}

// A specific instance of an event happening at a time.
#[derive(Debug, Copy, Clone)]
pub struct ReactionEvent {
    pub r1_loc: usize,         // Index in components.latest_states.
    pub r2_loc: Option<usize>, // Index in components.latest_states.
    pub rxn_idx: usize,        // Which reaction (from components.all_reactions).
    pub t: f32,                // When the reaction will fire.
    pub t_issued: f32,         // When the reaction was issued.
}

impl ReactionDescription {
    pub fn all_states(&self) -> Vec<&str> {
        let mut states: Vec<&str> = Vec::new();
        states.push(&self.r1);
        states.push(&self.p1);
        if self.r2.is_some() {
            states.push(self.r2.as_ref().unwrap());
        }
        if self.p2.is_some() {
            states.push(self.p2.as_ref().unwrap());
        }
        states
    }
}