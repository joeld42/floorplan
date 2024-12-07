use bevy::{prelude::* };
use bevy_egui::{
    //egui::{self, Color32},
    egui::{self},
    EguiContexts,
    //EguiPlugin
    };

#[derive(Default, Resource)]
pub struct OccupiedScreenSpace {
    pub left: f32,
    // pub top: f32,
    // pub right: f32,
    pub bottom: f32,
}

use super::floorplan;

pub fn ui_example_system(
    mut is_last_selected: Local<bool>,
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut floorplan: ResMut<floorplan::Floorplan>,
    mut state: ResMut<floorplan::InteractionState>,
) {
    let ctx = contexts.ctx_mut();

    occupied_screen_space.left = egui::SidePanel::left("left_panel")
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

            // Mode button Select
            if ui
                .add(egui::widgets::Button::new("Select")
                .selected( state.mode == floorplan::InteractionMode::Select ))
                .clicked()
            {
                state.mode = floorplan::InteractionMode::Select;
            }

            ui.label("Create");
            if ui
                .add(egui::widgets::Button::new("WALL").selected(!*is_last_selected))
                .clicked()
            {
                *is_last_selected = false;

                if state.selected_anchors.len() >= 2 {
                    let a = state.selected_anchors[0];
                    let b = state.selected_anchors[1];
                    floorplan.walls.push( floorplan::Wall { anchor_a : a, anchor_b : b, ..default() });
                }

            }
            if ui
                .add(egui::widgets::Button::new("Another button").selected(*is_last_selected))
                .clicked()
            {
                *is_last_selected = true;
            }
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
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
    occupied_screen_space.bottom = egui::TopBottomPanel::bottom("status_bar")
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Status Bar");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();
}
