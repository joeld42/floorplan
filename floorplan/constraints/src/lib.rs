pub use glam::{ Vec2 };

#[derive(Copy,Clone)]
pub enum PinMode
{
    Unpinned,
    PinX,
    PinY,
    PinXY,
}

#[derive(Copy,Clone)]
pub struct AnchorPoint
{
    pub p : Vec2,
    pub pin : PinMode,
}

#[derive(Default)]
pub struct ConstraintSystem
{
    pub anchors : Vec<AnchorPoint>,
    constraints : Vec<Constraint>,
}

// impl Default for ConstraintSystem {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl ConstraintSystem
{
    pub fn new() -> Self {
        Self { anchors: Vec::new(), constraints : Vec::new() }
    }

    // Note: in a larger system I'd probably use slotmap handles for these instead of
    // raw indices. Doing it this way for speed/simplicity.
    //
    // todo: way to wrap index so it's typesafe?
    pub fn add_anchor( &mut self, p : Vec2 ) -> usize {
        let index = self.anchors.len();
        self.anchors.push( AnchorPoint { p : p, pin : PinMode::Unpinned });
        index
    }

    // If target_len is none, will use the current length between the anchors
    pub fn add_constraint_fixed_len( &mut self, a : usize, b : usize, target_len : Option<f32> ) {

        let target_len = match target_len {
            Some( len ) => len,
            None => (self.anchors[ b ].p - self.anchors[ a ].p).length(),
        };

        self.constraints.push( Constraint::FixedLength( FixedLengthConstraint { anc_a : a, anc_b : b, target_len : target_len }) );
    }

    pub fn eval_system( &mut self ) {
        // let steps = 100;
        // let base_str = 5.0;

        let steps = 1;
        let base_str = 1.0;

        let str = base_str / (steps as f32);

        for substep in 0..steps {

            for cons in self.constraints.iter() {

                match cons {
                    Constraint::DummyConstraint => {}
                    Constraint::FixedLength( fixed_len ) => {

                        // note: could use split_at_mut here to get two slices of self.anchors but
                        // I like this better. Might be faster to just pass and return copies instead
                        // of referencing at all, todo investigate afterward
                        let mut anc_a = self.anchors[ fixed_len.anc_a ];
                        let mut anc_b = self.anchors[ fixed_len.anc_b ];
                        fixed_len.eval( &mut anc_a, &mut anc_b, str  );
                        self.anchors[fixed_len.anc_a].p = anc_a.p;
                        self.anchors[fixed_len.anc_b].p = anc_b.p;

                    }
                }
            }
        }
    }

}

// ====== [ FixedLengthConstraint ]==============================

struct FixedLengthConstraint {
    anc_a : usize,
    anc_b : usize,
    target_len : f32,
}

impl FixedLengthConstraint {

    fn eval( self : &Self, anc_a : &mut AnchorPoint, anc_b : &mut AnchorPoint, str : f32  ) {
        let dir = anc_b.p - anc_a.p;
        let curr_d = dir.length();
        let diff = curr_d - self.target_len;

        let dir = dir.normalize() * str * 0.5f32 * diff;

        // modify anchors towards target length
        anc_a.p = anc_a.p + dir;
        anc_b.p = anc_b.p - dir;
    }
}

// ============================================

enum Constraint {
    DummyConstraint,
    FixedLength( FixedLengthConstraint ),
}






// ============================================

pub fn add_vec2( left : Vec2, right: Vec2 ) -> Vec2 {
    return left + right;
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
