//use bevy::{prelude::*, window::PrimaryWindow };
use bevy::{prelude::*, scene::SceneBundle };

use bevy::render::camera::ScalingMode;
use bevy_egui::{
    //egui::{self, Color32},
    // egui::{self},
    // EguiContexts,
    EguiPlugin};

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

use bevy_vello::{ prelude::*, VelloPlugin };

use floorplan::{Floorplan, FloorplanUndoStack};

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
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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


    //let walls_src = asset_server.load("walls.glb#Scene0");




}

fn update_constraints( mut floorplan : ResMut<floorplan::Floorplan> )
{
    // update the constraint solver
    floorplan.csys.eval_system();
}



