use core::f32;

use bevy::{prelude::* };
use bevy::input::mouse::MouseButtonInput;

use super::floorplan;

// This file contains interaction logic for dragging/selecting

pub fn cursor_events(
    mut evr_cursor: EventReader<CursorMoved>,
    buttons: Res<ButtonInput<MouseButton>>,
    q_camera: Query<(&Camera, &Camera2d, &GlobalTransform)>,
    mut q_pancam : Query<&mut bevy_pancam::PanCam>,
    mut floorplan : ResMut<floorplan::Floorplan>,
    mut state : ResMut<floorplan::InteractionState>,
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

            // disable camera panning if hovering
            let mut pancam = q_pancam.single_mut();
            pancam.enabled = state.hover_anchor.is_none();

        }

        match state.mode {
            floorplan::InteractionMode::Adjust => {
                state.drag_anchor = if buttons.pressed( MouseButton::Left ) {
                    state.hover_anchor
               } else {
                   None
               };
            }

            floorplan::InteractionMode::SelectAnchors => { }
            floorplan::InteractionMode::SelectWalls => { }
        }


        // Adjust drag anchor
        if let Some(drag_anchor) = state.drag_anchor {
            floorplan.csys.anchors[drag_anchor].p = world_pos;
        }
    }
}

pub fn mouse_button_events(
    mut floorplan : ResMut<floorplan::Floorplan>,
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    mut state : ResMut<floorplan::InteractionState>,
) {
    use bevy::input::ButtonState;

    if (state.egui_active) {
        return;
    }


    for ev in mousebtn_evr.read() {

        match ev.state {
            ButtonState::Pressed => {
                //println!("Mouse button press: {:?}", ev.button);

                // tmp: figure out better interaction for creating anchors
                if ev.button == MouseButton::Right {

                    let new_anc = floorplan.csys.add_anchor( state.world_cursor );
                    state.selected_anchors.push( new_anc );
                }


                if ev.button == MouseButton::Left {

                    let mut did_select = false;

                    if state.mode == floorplan::InteractionMode::SelectAnchors {

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

                    if state.mode == floorplan::InteractionMode::SelectWalls {


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
                }

            }


            ButtonState::Released => {
                // println!("Mouse button release: {:?}", ev.button);
            }
        }
    }
}
