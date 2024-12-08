use core::f32;

use bevy::ecs::world;
use bevy::{prelude::* };
use bevy::input::mouse::MouseButtonInput;

use super::floorplan;

use constraints::{ PinMode };

// This file contains interaction logic for dragging/selecting

#[derive(Copy, Clone,PartialEq)]
pub enum InteractionMode {
    Adjust,
    Create,
    SelectAnchors,
    SelectWalls,
}
impl Default for InteractionMode {
    fn default() -> Self {
        InteractionMode::Adjust
    }
}

#[derive(Default)]
pub struct CreateModeInteractionState {
    // for create mode
    pub is_dragging: bool,
    pub drag_start: Vec2,
    pub drag_end: Vec2,
    pub anc_start : Option<usize>,
    pub anc_end : Option<usize>,
}

#[derive(Resource, Default)]
pub struct InteractionState {
    pub mode : InteractionMode,
    pub world_cursor : Vec2,
    pub hover_anchor : Option<usize>,
    pub drag_anchor : Option<usize>,

    pub create : CreateModeInteractionState,

    pub selected_anchors : Vec<usize>,
    pub selected_walls : Vec<usize>,

    pub left_panel: f32,
    pub egui_active : bool,

    // pub fn clear_selection( &mut self ) {
    //     self.hover_anchor = None;
    //     self.drag_anchor = None;
    //     self.selected_anchors.clear();
    //     self.selected_walls.clear();
    // }
}



pub fn cursor_events(
    mut evr_cursor: EventReader<CursorMoved>,
    buttons: Res<ButtonInput<MouseButton>>,
    q_camera: Query<(&Camera, &Camera2d, &GlobalTransform)>,
    mut q_pancam : Query<&mut bevy_pancam::PanCam>,
    mut floorplan : ResMut<floorplan::Floorplan>,
    mut state : ResMut<InteractionState>,
) {
    for ev in evr_cursor.read() {

        let ( cam, _, cam_transform ) = q_camera.single();

        let Some(world_pos) = cam.viewport_to_world_2d( cam_transform, ev.position ) else {
            return
        };

        // Somewhat hacky way to ignore egui_events.
        state.egui_active = ev.position.x < state.left_panel;
        if state.egui_active {
            return;
        }

        // update the world cursor
        state.world_cursor = world_pos;

        // println!(
        //     "New cursor position: X: {}, Y: {}, world {world_pos} in Window ID: {:?}",
        //     ev.position.x, ev.position.y, ev.window
        // );

        // TODO: screen space dist instead of world space?

        if state.drag_anchor.is_none() {

            // update the hover anchor if we're not currently dragging
            let mut hover_anc:  Option<usize> = None;
            for (ndx, anc) in floorplan.csys.anchors.iter().enumerate() {
                if anc.p.distance(world_pos) < 5.0 {
                    hover_anc = Some(ndx)
                }
            }
            state.hover_anchor = hover_anc;
        }

        // disable camera panning if hovering (fixme: this is probably not
        // the best place to handle this)
        let mut pancam = q_pancam.single_mut();
        //pancam.enabled = state.hover_anchor.is_none();

        pancam.enabled =  match state.mode {
            InteractionMode::Adjust => {
                state.hover_anchor.is_none()
            }
            _ => { false}
        };


        match state.mode {
            InteractionMode::Adjust => {
                state.drag_anchor = if buttons.pressed( MouseButton::Left ) {
                    state.hover_anchor
               } else {
                   None
               };
            }

            InteractionMode::Create => { }

            InteractionMode::SelectAnchors => { }
            InteractionMode::SelectWalls => { }
        }


        // Adjust drag anchor
        if let Some(drag_anchor) = state.drag_anchor {
            let pin = floorplan.csys.anchors[drag_anchor].pin;
            if pin != PinMode::PinXY {
                let p = floorplan.csys.anchors[drag_anchor].p;
                floorplan.csys.anchors[drag_anchor].p = match pin {
                    PinMode::Unpinned => world_pos,
                    PinMode::PinX => Vec2::new( p.x, world_pos.y ),
                    PinMode::PinY => Vec2::new( world_pos.x, p.y  ),
                    _ => unreachable!(), // Don't try to drag fully pinned anchors
                }
            }
        }

        // Adjust create-drag ghost line
        if state.mode == InteractionMode::Create && state.create.is_dragging {
            state.create.drag_end = world_pos;
            state.create.anc_end = floorplan.find_anchor( state.create.drag_end, 5.0);
        }

    }
}

pub fn mouse_button_events(
    mut floorplan : ResMut<floorplan::Floorplan>,
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    mut state : ResMut<InteractionState>,
) {
    use bevy::input::ButtonState;

    if state.egui_active {
        return;
    }


    for ev in mousebtn_evr.read() {

        match ev.state {
            ButtonState::Pressed => {
                //println!("Mouse button press: {:?}", ev.button);



                if ev.button == MouseButton::Left {

                    let mut did_select = false;

                    if state.mode == InteractionMode::Create {

                        state.create.is_dragging = true;
                        state.create.drag_start = state.world_cursor;
                        state.create.drag_end = state.world_cursor;
                        state.create.anc_start = floorplan.find_anchor( state.create.drag_start, 5.0);
                        state.create.anc_end = state.create.anc_start;
                    }


                    if state.mode == InteractionMode::SelectAnchors {

                        if let Some(hover_anchor) = state.hover_anchor {
                            did_select = true;

                            // toggle selected
                            if state.selected_anchors.contains( &hover_anchor ) {

                                // Remove hover_anchor from state.selected_anchors
                                state.selected_anchors.retain(|x| *x != hover_anchor );

                            } else {
                                state.selected_anchors.push( hover_anchor );
                            }
                        }

                        if !did_select {
                            state.selected_anchors.clear();
                        }
                    }

                    if state.mode == InteractionMode::SelectWalls {


                        let mut did_select = false;

                        let mut best_d = f32::MAX;
                        let mut closest_wall = None;
                        for i in 0..floorplan.walls.len() {

                            let d = floorplan.distance_to_wall( i, state.world_cursor );
                            if (d < 5.0) && (d < best_d) {
                                best_d = d;
                                closest_wall = Some( i )
                            }
                        }

                        if let Some(closest_wall) = closest_wall {

                            did_select = true;

                            // toggle selected
                            if state.selected_walls.contains( &closest_wall ) {

                                // Remove hover_anchor from state.selected_walls
                                state.selected_walls.retain(|x| *x != closest_wall );

                            } else {
                                state.selected_walls.push( closest_wall );
                            }
                        }

                        if !did_select {
                            state.selected_walls.clear();
                        }
                    }
                } else if ev.button == MouseButton::Right {

                    // Create mode, cancel dragging wall
                    if state.mode == InteractionMode::Create {
                        state.create.is_dragging = false;
                    }
                }
            }




            ButtonState::Released => {
                // println!("Mouse button release: {:?}", ev.button);


                // if ev.button == MouseButton::Right {

                //     // RMB is a shortcut to create an anchor
                //     if state.mode == InteractionMode::Create {
                //         let _new_anc = floorplan.csys.add_anchor( state.world_cursor );

                //         state.create.is_dragging = false;
                //     }
                // }
                if ev.button == MouseButton::Left {

                    if state.mode == InteractionMode::Create && state.create.is_dragging {

                        state.create.is_dragging = false;
                        state.create.drag_end = state.world_cursor;

                        state.create.anc_end = floorplan.find_anchor( state.create.drag_end, 5.0);

                        // Check minumum distance, otherwise just create an anchor
                        if state.create.drag_start.distance( state.create.drag_end) < 10.0 {

                            // just create an anchor if there isn't one there
                            if state.create.anc_start.is_none() && state.create.anc_end.is_none() {
                                let ctr = (state.create.drag_start + state.create.drag_end) * 0.5;
                                println!("Create anchor {:?}", ctr );
                                let _new_anc = floorplan.csys.add_anchor( ctr );
                            }

                        } else {

                            // Create the wall
                            create_wall( &mut floorplan, &state.create );
                        }
                    }
                }
            }
        }
    }
}

fn create_wall( floorplan : &mut floorplan::Floorplan, create : &CreateModeInteractionState )
{

    // Use or create an anchor for A
    let anc_start = create.anc_start.unwrap_or_else(|| floorplan.csys.add_anchor( create.drag_start ) );
    let anc_end = create.anc_end.unwrap_or_else(|| floorplan.csys.add_anchor( create.drag_end ) );

    // make sure wall doesn't already exist
    let existing_wall = floorplan.find_wall(anc_start, anc_end);
    if existing_wall.is_none() {
        println!("Create wall {:?} {:?}", create.anc_start, create.anc_end  );
        floorplan.walls.push( floorplan::Wall { anchor_a : anc_start, anchor_b : anc_end, ..default() });
    } else {
        println!("Wall already exists {} {}", anc_start, anc_end  );
    }

}
