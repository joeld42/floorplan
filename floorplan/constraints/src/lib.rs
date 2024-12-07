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

    pub fn add_constraint_parallel( &mut self, a : usize, b : usize, c : usize, d : usize ) {

        self.constraints.push(
            Constraint::Parallel( ParallelConstraint { anc_a : a, anc_b : b, anc_c : c, anc_d : d } )
        );

    }

    pub fn add_constraint_angle( &mut self, a : usize, b : usize, c : usize, target_ang : Option<f32> )
    {
        let target_ang = match target_ang {
            Some( len ) => len,
            None => {
                let ba = (self.anchors[a].p - self.anchors[b].p).normalize();
                let bc = (self.anchors[c].p - self.anchors[b].p).normalize();
                let dot = ba.dot( bc );

                // angle between BC and BC
                dot.acos()
            }
        };

        println!("Target angle {}", target_ang.to_degrees() );

        self.constraints.push( Constraint::Angle( AngleConstraint { anc_a : a, anc_b : b, anc_c : c, target_angle : target_ang }));
    }

    pub fn eval_system( &mut self ) {
        let steps = 100;
        let base_str = 5.0;

        // let steps = 1;
        // let base_str = 1.0;

        let str = base_str / (steps as f32);

        for _substep in 0..steps {

            for cons in self.constraints.iter() {

                match cons {
                    Constraint::DummyConstraint => {}
                    Constraint::FixedLength( fixed_len ) => {

                        // note: could use split_at_mut here to get two slices of self.anchors but
                        // I like this better. Might be faster to just pass and return copies or just the points
                        // instead of referencing at all the anchors, todo investigate afterward
                        let mut anc_a = self.anchors[ fixed_len.anc_a ];
                        let mut anc_b = self.anchors[ fixed_len.anc_b ];
                        fixed_len.eval( &mut anc_a, &mut anc_b, str  );
                        self.anchors[fixed_len.anc_a].p = anc_a.p;
                        self.anchors[fixed_len.anc_b].p = anc_b.p;

                    }

                    Constraint::Parallel( parallel ) => {
                        let mut anc_a = self.anchors[ parallel.anc_a ];
                        let mut anc_b = self.anchors[ parallel.anc_b ];
                        let mut anc_c = self.anchors[ parallel.anc_c ];
                        let mut anc_d = self.anchors[ parallel.anc_d ];
                        parallel.eval( &mut anc_a, &mut anc_b, &mut anc_c, &mut anc_d, str  );
                        self.anchors[parallel.anc_a].p = anc_a.p;
                        self.anchors[parallel.anc_b].p = anc_b.p;
                        self.anchors[parallel.anc_c].p = anc_c.p;
                        self.anchors[parallel.anc_d].p = anc_d.p;
                    }

                    Constraint::Angle( angle) => {
                        let mut anc_a = self.anchors[ angle.anc_a ];
                        let mut anc_b = self.anchors[ angle.anc_b ];
                        let mut anc_c = self.anchors[ angle.anc_c ];
                        angle.eval( &mut anc_a, &mut anc_b, &mut anc_c, str );
                        self.anchors[angle.anc_a].p = anc_a.p;
                        self.anchors[angle.anc_b].p = anc_b.p;
                        self.anchors[angle.anc_c].p = anc_c.p;

                    }
                }
            }
        }
    }

}

// ====== [ FixedLengthConstraint ]==============================
// Constrains AB to be the length target_len
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
// Rotation helpers
pub trait Vec2RotationHelpers {
    fn rotate_around_point( &self, center : Vec2, ang_radians : f32 ) -> Vec2;
}
impl Vec2RotationHelpers for Vec2 {
    fn rotate_around_point( &self, center : Vec2, ang_radians : f32 ) -> Vec2 {
        let p2 = *self - center;
        let s = ang_radians.sin();
        let c = ang_radians.cos();
        let pr = Vec2::new( p2.x*c - p2.y*s, p2.x*s + p2.y*c );

        // rotated result
        center + pr
    }
}


// ====== [ Parallel Constraint ]==============================
// Constrains AB to be parallel to CD
struct ParallelConstraint {
    anc_a : usize,
    anc_b : usize,
    anc_c : usize,
    anc_d : usize,
}

impl ParallelConstraint {

    // might be cleaner to do this by halves? eval AB, and then CD?
    fn eval( self : &Self,
        a1 : &mut AnchorPoint, b1 : &mut AnchorPoint,
        a2 : &mut AnchorPoint, b2 : &mut AnchorPoint,
        str : f32  ) {

            // this is some weird atan2 syntax
            let ang1 = ( b1.p.y - a1.p.y).atan2( b1.p.x - a1.p.x );
            let ang2 = ( b2.p.y - a2.p.y).atan2( b2.p.x - a2.p.x );

            let ang_diff = ang2 - ang1;
            let ang = ang_diff * 0.5 * str;

            let ctr1 = (a1.p + b1.p) * 0.5;
            a1.p = a1.p.rotate_around_point( ctr1, ang );
            b1.p = b1.p.rotate_around_point( ctr1, ang );

            let ctr2 = (a2.p + b2.p) * 0.5;
            a2.p = a2.p.rotate_around_point( ctr2, -ang );
            b2.p = b2.p.rotate_around_point( ctr2, -ang );
    }
}

// ====== [ Angle Constraint ]==============================
// Constrains the angle ABC to be a target angle
struct AngleConstraint {
    anc_a : usize,
    anc_b : usize,
    anc_c : usize,
    target_angle : f32, // in radians
}

impl AngleConstraint {

    fn eval( self : &Self,
        anc_a : &mut AnchorPoint,
        anc_b : &mut AnchorPoint,
        anc_c : &mut AnchorPoint,
         str : f32  ) {


            let ba = (anc_a.p - anc_b.p).normalize();
            let bc = (anc_c.p - anc_b.p).normalize();

            let dot = ba.dot( bc );
            let ang_curr = dot.acos();

            let ang_diff = ang_curr - self.target_angle;

            let ang = ang_diff * 0.5 * str;

            anc_a.p = anc_a.p.rotate_around_point( anc_b.p, -ang );
            anc_c.p = anc_c.p.rotate_around_point( anc_b.p, ang );

    }
}


// ============================================

enum Constraint {
    DummyConstraint,
    FixedLength( FixedLengthConstraint ),
    Parallel( ParallelConstraint ),
    Angle( AngleConstraint ),
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
