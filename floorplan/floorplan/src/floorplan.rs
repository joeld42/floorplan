
use bevy::{prelude::* };
use constraints::{ add, add_vec2, ConstraintSystem };

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

#[derive(Resource, Default)]
pub struct InteractionState {
    pub hover_anchor : Option<usize>,
    pub drag_anchor : Option<usize>,
}


