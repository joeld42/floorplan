
use bevy::{prelude::* };
use constraints::{ ConstraintSystem };

// Ended up not using this
#[derive(Copy,Clone)]
pub enum WallStyle
{
    Interior,
    Exterior
}

impl Default for WallStyle {
    fn default() -> Self {
        WallStyle::Interior
    }
}


#[derive(Copy,Clone)]
pub struct Wall
{
    pub anchor_a : usize,
    pub anchor_b : usize,
    pub _style : WallStyle,
}

impl Default for Wall {
    fn default() -> Self {
        Self {
            anchor_a: 0,
            anchor_b: 0,
            _style: WallStyle::default(),
        }
    }
}

pub struct UndoCheckpoint {
    pub op_name : String,
    pub floorplan : Floorplan,
    pub adjust : bool
}

#[derive(Resource, Default)]
pub struct FloorplanUndoStack
{
    pub stack : Vec<UndoCheckpoint>,
}

impl FloorplanUndoStack
{
    pub fn push_before_op( &mut self, name : &str, floorplan : &Floorplan)
    {
        self.stack.push( UndoCheckpoint { op_name : String::from(name), floorplan: floorplan.clone(), adjust : false } );
    }

    pub fn push_before_adjust( &mut self, floorplan : &Floorplan)
    {
        self.stack.push( UndoCheckpoint { op_name : String::from("Adjust"), floorplan: floorplan.clone(), adjust : true } );
    }

    pub fn is_top_adjust( &self ) -> bool {

        // ?? the if let should cover this??
        if self.stack.len() == 0 {
            return false
        }

        if let Some(top) = self.stack.last() {
            top.adjust
        } else {
            false
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct Floorplan
{
    pub csys : ConstraintSystem,
    pub walls : Vec<Wall>,
}

impl Floorplan
{

    pub fn distance_to_wall( &self, wall_ndx : usize, p : Vec2 ) -> f32 {
        let pa = self.csys.anchors[ self.walls[ wall_ndx ].anchor_a ].p;
        let pb = self.csys.anchors[ self.walls[ wall_ndx ].anchor_b ].p;

        let l2 = (pa - pb).length_squared();
        if  l2 < f32::EPSILON {
            // endpoints the same, just return distance to the first
            p.distance( pa )
        } else {
            let t = ((p - pa).dot( pb - pa ) / l2).clamp( 0.0, 1.0 );
            let proj = pa + t * (pb - pa );

            // distance to closest point on segment
            proj.distance( p )
        }
    }

    pub fn find_wall( &self, a : usize, b : usize ) -> Option<Wall> {
        self.walls.iter().find( |wall| {
            (wall.anchor_a == a && wall.anchor_b == b) ||
            (wall.anchor_a == b && wall.anchor_b == a)
        }).cloned()
    }

    // Finds the closest anchor within 'threshold' distance
    pub fn find_anchor( &self, pos : Vec2, threshold : f32 ) -> Option<usize> {
        let mut best_d = f32::MAX;
        let mut closest_anc = None;
        for (ndx, anc) in self.csys.anchors.iter().enumerate() {
            let d = anc.p.distance(pos);
            if (d < threshold) && (d < best_d) {
                closest_anc = Some(ndx);
                best_d = d;
            }
        }
        // result
        closest_anc
    }

    pub fn make_starter_floorplan() -> Floorplan {
        let mut floorplan = Floorplan::default();

        let a = floorplan.csys.add_anchor( Vec2::new( -100.0, -100.0 ));
        let b = floorplan.csys.add_anchor( Vec2::new(  100.0, -100.0 ));
        let c = floorplan.csys.add_anchor( Vec2::new(  100.0, 140.0 ));
        let d = floorplan.csys.add_anchor( Vec2::new(  -100.0, 80.0 ));

        floorplan.walls.push( Wall { anchor_a : a, anchor_b : b, ..default() });
        floorplan.walls.push( Wall { anchor_a : b, anchor_b : c, ..default() });
        floorplan.walls.push( Wall { anchor_a : c, anchor_b : d, ..default() });
        floorplan.walls.push( Wall { anchor_a : d, anchor_b : a, _style : WallStyle::Exterior });

        floorplan.csys.add_constraint_fixed_len( a, d, None );

        // result
        floorplan
    }


    pub fn copy_from ( &mut self, other : Floorplan )
    {
        self.csys = other.csys.clone();
        self.walls = other.walls.clone();
    }

}






