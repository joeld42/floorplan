use std::default;

use bevy::{prelude::* };
use bevy_egui::{
    //egui::{self, Color32},
    egui::{self},
    EguiContexts,
    //EguiPlugin
    };

use constraints::{ Constraint, AnchorPoint, PinMode };

use crate::floorplan::{Floorplan, FloorplanUndoStack};

use super::floorplan;
use super::interaction::{InteractionMode, InteractionState};

pub fn ui_example_system(
    mut contexts: EguiContexts,
    mut floorplan: ResMut<floorplan::Floorplan>,
    mut state: ResMut<InteractionState>,
    mut undo: ResMut<FloorplanUndoStack>,
) {
    let ctx = contexts.ctx_mut();

    state.left_panel = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {

            ui.label("Mode");

            // Mode button Adjust
            if ui
                .add(egui::widgets::Button::new("Adjust")
                .selected( state.mode == InteractionMode::Adjust ))
                .clicked()
            {
                state.mode = InteractionMode::Adjust;
            }

            // Mode button Adjust
            if ui
                .add(egui::widgets::Button::new("Create")
                .selected( state.mode == InteractionMode::Create ))
                .clicked()
            {
                state.mode = InteractionMode::Create;
            }

            // Mode button Select Walls
            if ui
                .add(egui::widgets::Button::new("Select Walls")
                .selected( state.mode == InteractionMode::SelectWalls ))
                .clicked()
            {
                state.mode = InteractionMode::SelectWalls;
                state.selected_anchors.clear();
            }

            // Mode button Select Anchors
            if ui
                .add(egui::widgets::Button::new("Select Anchors")
                .selected( state.mode == InteractionMode::SelectAnchors ))
                .clicked()
            {
                state.mode = InteractionMode::SelectAnchors;
                state.selected_walls.clear();
            }

            ui.label("Constraints");


            // Fixed Length
            //let can_add_length_constraint = state.selected_anchors.len() == 2;

            let can_add_length_constraint = match state.mode {
                InteractionMode::SelectAnchors => state.selected_anchors.len() == 2,
                InteractionMode::SelectWalls => state.selected_walls.len() == 1,
                _ => false,
            };

            // TODO: check there is not already a constraint
            if ui
                .add_enabled(can_add_length_constraint,
                    egui::widgets::Button::new("Fixed Length") )
                .clicked()
            {
                println!("Create Fixed Len constraint pressed");

                // Fixme check if constraint is already there
                if state.mode == InteractionMode::SelectAnchors
                {
                    undo.push_before_op( "Fixed Len Constraint", &floorplan );
                    floorplan.csys.add_constraint_fixed_len(
                        state.selected_anchors[0], state.selected_anchors[1], None );

                } else {
                    undo.push_before_op( "Fixed Len Constraint", &floorplan );

                    let wall = floorplan.walls[ state.selected_walls[0] ];
                    floorplan.csys.add_constraint_fixed_len( wall.anchor_a, wall.anchor_b, None );
                }
            }

            // Parallel walls
            let can_add_parallel_constraint = state.selected_walls.len() == 2;
            // TODO: check there is not already a constraint
            if ui
                .add_enabled(can_add_parallel_constraint,
                    egui::widgets::Button::new("Parallel") )
                .clicked()
            {

                let wall_a = floorplan.walls[ state.selected_walls[0] ];
                let wall_b = floorplan.walls[ state.selected_walls[1] ];

                let a = wall_a.anchor_a;
                let b = wall_a.anchor_b;

                let mut c = wall_b.anchor_a;
                let mut d = wall_b.anchor_b;

                // see if AB -> CD or AB -> DC start off closer to parallel
                let ab = (floorplan.csys.anchors[b].p - floorplan.csys.anchors[a].p).normalize();
                let cd = (floorplan.csys.anchors[d].p - floorplan.csys.anchors[c].p).normalize();
                if ab.dot( cd ) < 0.0 {
                    (d, c) = (c, d);
                }
                //println!("AB dot CD is {}", dot );

                undo.push_before_op( "Parallel Constraint", &floorplan );
                floorplan.csys.add_constraint_parallel( a,b,c,d );
            }

            // Fixed Angle
            let mut can_add_angle_constraint = state.selected_walls.len() == 2;

            let mut shared_anchor =0;
            if can_add_angle_constraint {

                // make sure exactly one anchor is shared
                let wall_a = floorplan.walls[ state.selected_walls[0] ];
                let wall_b =floorplan.walls[ state.selected_walls[1] ];

                // find the shared anchor between walls
                shared_anchor = if wall_a.anchor_a == wall_b.anchor_a || wall_a.anchor_a == wall_b.anchor_b {
                    wall_a.anchor_a
                } else if wall_a.anchor_b == wall_b.anchor_a || wall_a.anchor_b == wall_b.anchor_b {
                    wall_a.anchor_b
                } else {
                    can_add_angle_constraint = false;
                    0
                };
            }


            // TODO: check there is not already a constraint
            if ui
                .add_enabled(can_add_angle_constraint,
                    egui::widgets::Button::new("Angle") )
                .clicked()
            {
                let wall_a = floorplan.walls[ state.selected_walls[0] ];
                let wall_b =floorplan.walls[ state.selected_walls[1] ];

                let anc1 = if wall_a.anchor_a == shared_anchor {
                    wall_a.anchor_b
                } else {
                    wall_a.anchor_a
                };

                let anc2 = if wall_b.anchor_a == shared_anchor {
                    wall_b.anchor_b
                } else {
                    wall_b.anchor_a
                };

                let b = floorplan.csys.anchors[shared_anchor].p;
                let b1 = (floorplan.csys.anchors[anc1].p - b).normalize();
                let b2 = (b - floorplan.csys.anchors[anc2].p).normalize();
                let ccw = b1.x*b2.y - b1.y*b2.x;
                //println!("b1 cross b2 is {}", ccw );
                let (anc1,anc2) = if ccw < 0.0 {
                    (anc2, anc1)
                } else {
                    (anc1, anc2)
                };

                println!("make angle constraint for {} {} {}", anc1, shared_anchor, anc2 );
                undo.push_before_op( "Angle Constraint", &floorplan );
                floorplan.csys.add_constraint_angle( anc1, shared_anchor, anc2,None);
            }

            // Show panel for all selected anchors
            if state.mode == InteractionMode::SelectAnchors {
                for (ndx, anc) in floorplan.csys.anchors.iter_mut().enumerate() {
                    if state.selected_anchors.contains( &ndx ) {
                        edit_anchor_panel( ui, anc );
                    }
                }
            }

            // Show panel for all constraints on the currently selected stuff
            for cons in floorplan.csys.constraints.iter_mut() {

                // TODO: only show the constraints that have anchors or walls selected
                edit_constraint_pane( ui,  cons );
            }


            // Show current selection
            // ui.add(egui::Separator::default());
            // match state.mode {
            //     floorplan::InteractionMode::SelectAnchors => {
            //     },
            //     floorplan::InteractionMode::SelectWalls => {

            //         if state.selected_walls.len() > 1 {

            //             // show a lable with how many walls selected
            //             ui.label( format!("{} walls selected", state.selected_walls.len() ));
            //         } else if state.selected_walls.len() == 1 {
            //             // See if we have a length constraint on this wall
            //             let wall = floorplan.walls[ state.selected_walls[0] ];
            //             let cons = floorplan.csys.find_constraint( wall.anchor_a, wall.anchor_b );

            //             match cons {
            //                 Some( Constraint::FixedLength( cc_fixed ) ) => {
            //                     ui.label( format!("Wall Length: {}", cc_fixed.target_len ));
            //                 }
            //                 _ => {}
            //             }


            //         }
            //     },
            //     _ => {},
            // }


            ui.add(egui::Separator::default());

            ui.horizontal(|ui| {
                if ui
                    .add( egui::widgets::Button::new("Reset") )
                    .clicked()
                {
                    let orig = Floorplan::make_starter_floorplan();
                    floorplan.copy_from( orig );

                    undo.stack.clear();
                }

                if ui
                    .add( egui::widgets::Button::new("Clear") )
                    .clicked()
                {
                    let empty = Floorplan::default();
                    floorplan.copy_from( empty );

                    undo.stack.clear();
                }
            });

            let mut can_undo = true;

            let title = if let Some(top) = undo.stack.last() {
                format!("Undo {}", top.op_name )
            } else {
                can_undo = false;
                String::from("Undo")
            };


            if ui
                    .add_enabled(  can_undo, egui::widgets::Button::new(title ) )
                    .clicked()
                {

                    if let Some(entry) = undo.stack.pop() {
                        floorplan.copy_from( entry.floorplan );
                    }

                }



            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();

        // ============================================
        // Show selected items
        // ============================================

    // occupied_screen_space.right = egui::SidePanel::right("right_panel")
    //     .resizable(true)
    //     .show(ctx, |ui| {
    //         ui.label("Right resizeable panel");
    //         ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
    //     })
    //     .response
    //     .rect
    //     .width();
    // occupied_screen_space.top = egui::TopBottomPanel::top("top_panel")
    //     .resizable(true)
    //     .show(ctx, |ui| {
    //         ui.label("Top resizeable panel");
    //         ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
    //     })
    //     .response
    //     .rect
    //     .height();
    egui::TopBottomPanel::bottom("status_bar")
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Status Bar");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });
}

fn edit_anchor_panel( ui: &mut egui::Ui, anchor : &mut AnchorPoint )
{
    ui.add(egui::Separator::default());

    let mut pin_x = (anchor.pin==PinMode::PinX) || (anchor.pin==PinMode::PinXY);
    let mut pin_y = (anchor.pin==PinMode::PinY) || (anchor.pin==PinMode::PinXY);
    ui.horizontal(|ui| {
        ui.checkbox(&mut pin_x, "Pin X");
        ui.checkbox(&mut pin_y, "Pin Y");
    });

    anchor.pin = if pin_x && pin_y {
        PinMode::PinXY
    } else if pin_x {
        PinMode::PinX
    } else if pin_y {
        PinMode::PinY
    } else {
        PinMode::Unpinned
    };
}

fn edit_constraint_pane( ui: &mut egui::Ui, constraint : &mut Constraint )
{
    ui.add(egui::Separator::default());

    match constraint {

        Constraint::FixedLength( cc_fixed ) => {


            ui.label( "Fixed Length:" );
            if ui
                .add(egui::Slider::new(
                    &mut cc_fixed.target_len,
                    0.0..=500.0,
                ))
                .changed()
                {
                    //println!( "length changed...");
                };
        }
        Constraint::Angle( cc_ang ) => {
            let mut angle_deg = cc_ang.target_angle.to_degrees();
            ui.label( "Fixed Angle:" );
            if ui
                .add(egui::Slider::new(
                    &mut angle_deg,
                    3.0..=170.0,
                ))
                .changed()
                {
                    println!( "angle changed...");
                    cc_ang.target_angle = angle_deg.to_radians();
                };
        }
        _ => {}
    }
}
