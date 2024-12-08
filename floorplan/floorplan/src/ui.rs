use bevy::{prelude::* };
use bevy_egui::{
    //egui::{self, Color32},
    egui::{self},
    EguiContexts,
    //EguiPlugin
    };

use constraints::{ Constraint, AnchorPoint, PinMode };

use super::floorplan;
use super::interaction::{InteractionMode, InteractionState};

pub fn ui_example_system(
    mut contexts: EguiContexts,
    mut floorplan: ResMut<floorplan::Floorplan>,
    mut state: ResMut<InteractionState>,
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
                    floorplan.csys.add_constraint_fixed_len(
                        state.selected_anchors[0], state.selected_anchors[1], None );

                } else {
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

                floorplan.csys.add_constraint_parallel( a,b,c,d );
            }

            // Fixed Angle
            let can_add_angle_constraint = state.selected_anchors.len() == 3;
            // TODO: check there is not already a constraint
            if ui
                .add_enabled(can_add_angle_constraint,
                    egui::widgets::Button::new("Angle") )
                .clicked()
            {
                // fixme: don't depend on selection order here
                let a = state.selected_anchors[0];
                let b = state.selected_anchors[1];
                let c = state.selected_anchors[2];

                floorplan.csys.add_constraint_angle( a,b,c,None);
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
        _ => {}
    }
}
