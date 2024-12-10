//use bevy::{prelude::*, window::PrimaryWindow };
use bevy::{prelude::* };

use bevy::render::camera::ScalingMode;
use bevy_egui::{
    //egui::{self, Color32},
    // egui::{self},
    // EguiContexts,
    EguiPlugin};

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use bevy_vello::{ prelude::*, VelloPlugin };

use floorplan::{Floorplan, FloorplanUndoStack};
use constraints::{ PinMode };

mod diagram;
mod ui;
mod floorplan;
mod interaction;
mod preview;


fn main() {

    //let mut csys = ConstraintSystem::new();
    let floorplan = Floorplan::make_starter_floorplan();


    // TODO: split these systems into Plugins for tidyness
    App::new()
        //.insert_resource(WinitSettings::desktop_app())
        .insert_resource( floorplan )
        .init_resource::<FloorplanUndoStack>()
        .insert_resource(ClearColor(Color::srgb(0.176, 0.247, 0.431)))
        .init_resource::<interaction::InteractionState>()
        .init_resource::<preview::WallSet>()
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin::default())
        .add_plugins(EguiPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(bevy_pancam::PanCamPlugin)

        // Main app systems
        .add_systems(Startup, setup_system)
        .add_systems(Update, ui::ui_example_system)
        .add_systems(Update, diagram::render_diagram)
        .add_systems( Update, update_constraints )
        .add_systems( Update, interaction::cursor_events )
        .add_systems( Update, interaction::keyboard_input )
        .add_systems( Update, interaction::mouse_button_events )

        // preview systems
        .add_systems(Startup, preview::setup_preview)
        .add_systems( Update, preview::rebuild_floorplan )
        .add_systems( Update, preview::adjust_preview_camera )
        .add_event::<preview::RebuildFloorplan>()

        .run();
}


fn setup_system(
    mut commands: Commands,
) {

    let mut camera2d = Camera2dBundle {
        camera: Camera {
            //clear_color: ClearColorConfig::None,
            //clear_color: ClearColorConfig::Custom Color::srgb(0.176, 0.247, 0.431),
            order: 1,
            ..default()
        },
        ..Default::default()
     };

    // initial view 400m
    camera2d.projection.scaling_mode = ScalingMode::FixedVertical(400.0);
    //camera2d.transform = Transform::from_xyz(100.0, 200.0, 0.0);


    // Spawn diagram scene
    commands.spawn( (camera2d, bevy_pancam::PanCam::default() ) );
    commands.spawn(VelloSceneBundle::default());
}

fn update_constraints(
    mut state : ResMut<interaction::InteractionState>,
    mut undo: ResMut<FloorplanUndoStack>,
    mut floorplan : ResMut<floorplan::Floorplan>
)
{

    if let Some(drag_anchor) = state.drag_anchor {

        // hack..
        if !undo.is_top_adjust() {
            undo.push_before_adjust( &floorplan );
        }


        if state.solve_from_mousedown {

            // resize and store current anchors (should happen on mousedown)
            if state.anc_pos_mousedown.len() != floorplan.csys.anchors.len() {
                state.anc_pos_mousedown.resize( floorplan.csys.anchors.len(), Vec2::ZERO );

                for i in 0..floorplan.csys.anchors.len() {
                    state.anc_pos_mousedown[i] = floorplan.csys.anchors[i].p;
                }
            }

            // Restore anchors from mousedown
            for i in 0..floorplan.csys.anchors.len() {
                floorplan.csys.anchors[i].p = state.anc_pos_mousedown[i];
            }
        }

        let pin = floorplan.csys.anchors[drag_anchor].pin;
        if pin != PinMode::PinXY {
            let p = floorplan.csys.anchors[drag_anchor].p;
            floorplan.csys.anchors[drag_anchor].p = match pin {
                PinMode::Unpinned => state.world_cursor,
                PinMode::PinX => Vec2::new( p.x, state.world_cursor.y ),
                PinMode::PinY => Vec2::new( state.world_cursor.x, p.y  ),
                _ => unreachable!(), // Don't try to drag fully pinned anchors
            }
        }
        //println!("drag anchor is {}", drag_anchor );
    } else {
        // no drag anchor
        state.anc_pos_mousedown.clear();
    }


    // update the constraint solver
    floorplan.csys.eval_system();
}



