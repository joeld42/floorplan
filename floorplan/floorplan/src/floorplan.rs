
use bevy::{prelude::* };
use constraints::{ ConstraintSystem };

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
    pub style : WallStyle,
}

impl Default for Wall {
    fn default() -> Self {
        Self {
            anchor_a: 0,
            anchor_b: 0,
            style: WallStyle::default(),
        }
    }
}


#[derive(Resource, Default)]
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
}






