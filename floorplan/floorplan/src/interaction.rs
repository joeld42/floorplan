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

            floorplan::InteractionMode::Select => { }
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


    for ev in mousebtn_evr.read() {


        match ev.state {
            ButtonState::Pressed => {
                println!("Mouse button press: {:?}", ev.button);

                // tmp: figure out better interaction
                if ev.button == MouseButton::Right {

                    let new_anc = floorplan.csys.add_anchor( state.world_cursor );
                    state.selected_anchors.push( new_anc );
                }

                if (ev.button == MouseButton::Left) && (state.mode == floorplan::InteractionMode::Select) {

                    if let Some(hover_anchor) = state.hover_anchor {
                        // toggle selected
                        if state.selected_anchors.contains( &hover_anchor ) {

                            // Remove hover_anchor from state.selected_anchors
                            state.selected_anchors.retain(|x| *x != hover_anchor );

                        } else {
                            state.selected_anchors.push( hover_anchor );
                        }
                    }
                }

                // to keep things simple, a maximum of 2 anchors may be selected
                while state.selected_anchors.len() > 2 {
                    state.selected_anchors.remove( 0 );
                }

            }


            ButtonState::Released => {
                println!("Mouse button release: {:?}", ev.button);
            }
        }
    }
}

pub fn update_selection (
    floorplan : Res<floorplan::Floorplan>,
    mut state : ResMut<floorplan::InteractionState>,
) {

    // Update selected wall, if two anchors of a wall are selected,
    // then we consider the wall to be selected
    let mut sel_wall = None;
    if state.selected_anchors.len() == 2 {
        let a = state.selected_anchors[0];
        let b = state.selected_anchors[1];
        for (ndx, wall) in floorplan.walls.iter().enumerate() {
            if ( (wall.anchor_a == a) && (wall.anchor_b == b) ) ||
               ( (wall.anchor_b == a) && (wall.anchor_a == b) ) {
                sel_wall = Some(ndx);
                break;
            }
        }
    }
    state.selected_wall = sel_wall;
}
