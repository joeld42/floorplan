use bevy::{prelude::* };
use bevy_egui::{
    //egui::{self, Color32},
    egui::{self},
    EguiContexts,
    //EguiPlugin
    };

use constraints::{ Constraint };

#[derive(Default, Resource)]
pub struct OccupiedScreenSpace {
    pub left: f32,
    // pub top: f32,
    // pub right: f32,
    pub bottom: f32,
}

use super::floorplan;

pub fn ui_example_system(
    mut contexts: EguiContexts,
    //mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut floorplan: ResMut<floorplan::Floorplan>,
    mut state: ResMut<floorplan::InteractionState>,
) {
    let ctx = contexts.ctx_mut();

    state.left_panel = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {

            ui.label("Mode");

            // Mode button Adjust
            if ui
                .add(egui::widgets::Button::new("Adjust")
                .selected( state.mode == floorplan::InteractionMode::Adjust ))
                .clicked()
            {
                state.mode = floorplan::InteractionMode::Adjust;
            }

            // Mode button Select Walls
            if ui
                .add(egui::widgets::Button::new("Select Walls")
                .selected( state.mode == floorplan::InteractionMode::SelectWalls ))
                .clicked()
            {
                state.mode = floorplan::InteractionMode::SelectWalls;
                state.selected_anchors.clear();
            }

            // Mode button Select Anchors
            if ui
                .add(egui::widgets::Button::new("Select Anchors")
                .selected( state.mode == floorplan::InteractionMode::SelectAnchors ))
                .clicked()
            {
                state.mode = floorplan::InteractionMode::SelectAnchors;
                state.selected_walls.clear();
            }

            ui.label("Create");
            if ui
                .add(egui::widgets::Button::new("WALL"))
                .clicked()
            {
                if state.selected_anchors.len() >= 2 {
                    let a = state.selected_anchors[0];
                    let b = state.selected_anchors[1];
                    floorplan.walls.push( floorplan::Wall { anchor_a : a, anchor_b : b, ..default() });
                }

            }

            ui.label("Constraints");


            // Fixed Length
            //let can_add_length_constraint = state.selected_anchors.len() == 2;

            let can_add_length_constraint = match state.mode {
                floorplan::InteractionMode::SelectAnchors => state.selected_anchors.len() == 2,
                floorplan::InteractionMode::SelectWalls => state.selected_walls.len() == 1,
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
                if state.mode == floorplan::InteractionMode::SelectAnchors
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
                .add_enabled(can_add_parallel_constraint,
                    egui::widgets::Button::new("Angle") )
                .clicked()
            {
                // fixme: don't depend on selection order here
                let a = state.selected_anchors[0];
                let b = state.selected_anchors[1];
                let c = state.selected_anchors[2];

                floorplan.csys.add_constraint_angle( a,b,c,None);
            }


            // Show current selection
            ui.add(egui::Separator::default());
            match state.mode {
                floorplan::InteractionMode::SelectAnchors => {
                },
                floorplan::InteractionMode::SelectWalls => {

                    if state.selected_walls.len() > 1 {

                        // show a lable with how many walls selected
                        ui.label( format!("{} walls selected", state.selected_walls.len() ));
                    } else if state.selected_walls.len() == 1 {
                        // See if we have a length constraint on this wall
                        let wall = floorplan.walls[ state.selected_walls[0] ];
                        let cons = floorplan.csys.find_constraint( wall.anchor_a, wall.anchor_b );

                        match cons {
                            Some( Constraint::FixedLength( cc_fixed ) ) => {
                                ui.label( format!("Wall Length: {}", cc_fixed.target_len ));
                            }
                            _ => {}
                        }


                    }
                },
                _ => {},
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
