
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
}


#[derive(Copy, Clone,PartialEq)]
pub enum InteractionMode {
    Adjust,
    SelectAnchors,
    SelectWalls,
}
impl Default for InteractionMode {
    fn default() -> Self {
        InteractionMode::Adjust
    }
}

#[derive(Resource, Default)]
pub struct InteractionState {
    pub mode : InteractionMode,
    pub world_cursor : Vec2,
    pub hover_anchor : Option<usize>,
    pub drag_anchor : Option<usize>,

    pub selected_anchors : Vec<usize>,
    pub selected_walls : Vec<usize>,

    pub left_panel: f32,
    pub egui_active : bool,
}


