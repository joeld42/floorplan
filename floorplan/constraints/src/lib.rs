pub use glam::{ Vec2 };


// TODO make this a bitfield?
#[derive(Copy,Clone,PartialEq)]
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
    pub p_orig : Vec2,
    pub pin : PinMode,
}

#[derive(Default, Clone)]
pub struct ConstraintSystem
{
    pub anchors : Vec<AnchorPoint>,

    // these don't need to be pub, (and probably shouldn't be),
    // but I need to access them to draw the constraints.
    pub constraints : Vec<Constraint>,
}



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
        self.anchors.push( AnchorPoint { p : p, p_orig : p, pin : PinMode::Unpinned });
        index
    }

    pub fn find_constraint( &self, a : usize, b : usize ) -> Option<Constraint> {
        self.constraints.iter().find( |cc| {
            match cc {
                Constraint::FixedLength( cc_fixed ) => {
                    (cc_fixed.anc_a == a && cc_fixed.anc_b == b) ||
                    (cc_fixed.anc_a == b && cc_fixed.anc_b == a)
                }
                _ => false
            }

        }).cloned()
    }

    // If target_len is none, will use the current length between the anchors
    pub fn add_constraint_fixed_len( &mut self, a : usize, b : usize, target_len : Option<f32> ) {

        println!("Add constraint fixed len {} {} -- {:?}", a, b, target_len );

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

        // println!("Target angle {}", target_ang.to_degrees() );

        self.constraints.push( Constraint::Angle( AngleConstraint { anc_a : a, anc_b : b, anc_c : c, target_angle : target_ang }));
    }

    pub fn eval_system( &mut self ) {
        let steps = 100;
        let base_str = 5.0;

        // let steps = 1;
        // let base_str = 1.0;

        let str = base_str / (steps as f32);

        for _substep in 0..steps {

            // store orig pos
            for anc in self.anchors.iter_mut() {
                anc.p_orig = anc.p;
            }

            for cons in self.constraints.iter() {

                match cons {
                    //Constraint::DummyConstraint => {}
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

            // Apply pins
            for anc in self.anchors.iter_mut() {
                if anc.pin == PinMode::Unpinned {
                    continue;
                }

                anc.p = match anc.pin {
                    PinMode::PinX => Vec2::new( anc.p_orig.x, anc.p.y ),
                    PinMode::PinY => Vec2::new( anc.p.x, anc.p_orig.y ),
                    PinMode::PinXY => anc.p_orig,
                    _ => unreachable!()
                };
            }

        }
    }

}

// ====== [ FixedLengthConstraint ]==============================
// Constrains AB to be the length target_len
#[derive(Clone)]
pub struct FixedLengthConstraint {
    pub anc_a : usize,
    pub anc_b : usize,
    pub target_len : f32,
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
    fn rotate_around_point_lim( &self, center : Vec2, ang_radians : f32, lim : f32 ) -> Vec2;
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

    fn rotate_around_point_lim( &self, center : Vec2, ang_radians : f32, lim : f32 ) -> Vec2 {

        let p2 = self.rotate_around_point( center, ang_radians);
        if self.distance(p2) < lim {
            p2
        } else {

            let dir = (p2 - *self).normalize();
            *self + dir * lim
        }

    }
}


// ====== [ Parallel Constraint ]==============================
// Constrains AB to be parallel to CD
#[derive(Clone)]
pub struct ParallelConstraint {
    pub anc_a : usize,
    pub anc_b : usize,
    pub anc_c : usize,
    pub anc_d : usize,
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
#[derive(Clone)]
pub struct AngleConstraint {
    pub anc_a : usize,
    pub anc_b : usize,
    pub anc_c : usize,
    pub target_angle : f32, // in radians
}

impl AngleConstraint {

    fn eval( self : &Self,
        anc_a : &mut AnchorPoint,
        anc_b : &mut AnchorPoint,
        anc_c : &mut AnchorPoint,
         str : f32 ) {


            let ba = (anc_a.p - anc_b.p).normalize();
            let bc = (anc_c.p - anc_b.p).normalize();

            let dot = ba.dot( bc );
            let ang_curr = dot.acos();

            let ang_diff = ang_curr - self.target_angle;

            let ang = ang_diff * 0.5 * str;

            anc_a.p = anc_a.p.rotate_around_point_lim( anc_b.p, -ang, 0.1 );
            anc_c.p = anc_c.p.rotate_around_point_lim( anc_b.p, ang, 0.1 );

    }
}


// ============================================

#[derive(Clone)]
pub enum Constraint {
    //DummyConstraint,
    FixedLength( FixedLengthConstraint ),
    Parallel( ParallelConstraint ),
    Angle( AngleConstraint ),
}
