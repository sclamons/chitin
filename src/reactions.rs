#[derive(Debug, Clone, Copy)]
pub struct Reaction {
    pub r1_idx: usize,
    pub r2_idx: Option<usize>,
    pub p1_idx: usize,
    pub p2_idx: Option<usize>,
    pub rate: f32
}

#[derive(Debug)]
pub struct ReactionDescription {
    pub r1: String,
    pub r2: Option<String>,
    pub p1: String,
    pub p2: Option<String>,
    pub rate: f32
}

#[derive(Debug, Copy, Clone)]
pub struct ReactionEvent {
    pub r1_idx: usize,
    pub r2_idx: Option<usize>,
    pub rxn_idx: usize,
    pub t: f32,
    pub t_issued: f32,
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