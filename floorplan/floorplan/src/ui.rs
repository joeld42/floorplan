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


pub fn ui_example_system(
    mut is_last_selected: Local<bool>,
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
) {
    let ctx = contexts.ctx_mut();

    occupied_screen_space.left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Left resizeable panel");
            if ui
                .add(egui::widgets::Button::new("A button").selected(!*is_last_selected))
                .clicked()
            {
                *is_last_selected = false;
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
