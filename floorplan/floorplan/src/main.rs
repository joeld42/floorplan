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

use floorplan::Floorplan;

mod diagram;
mod ui;
mod floorplan;
mod interaction;

const CAMERA_TARGET: Vec3 = Vec3::ZERO;

#[derive(Resource, Deref, DerefMut)]
struct OriginalCameraTransform(Transform);


fn main() {


    //let mut csys = ConstraintSystem::new();
    let mut floorplan = Floorplan::default();

    let a = floorplan.csys.add_anchor( Vec2::new( -100.0, -100.0 ));
    let b = floorplan.csys.add_anchor( Vec2::new(  100.0, -100.0 ));
    let c = floorplan.csys.add_anchor( Vec2::new(  100.0, 120.0 ));
    let d = floorplan.csys.add_anchor( Vec2::new(  -100.0, 100.0 ));

    floorplan.walls.push( floorplan::Wall { anchor_a : a, anchor_b : b, ..default() });
    floorplan.walls.push( floorplan::Wall { anchor_a : b, anchor_b : c, ..default() });
    floorplan.walls.push( floorplan::Wall { anchor_a : c, anchor_b : d, ..default() });
    floorplan.walls.push( floorplan::Wall { anchor_a : d, anchor_b : a, style : floorplan::WallStyle::Exterior });

    //floorplan.csys.add_constraint_fixed_len( a, d, None );
    //floorplan.csys.add_constraint_parallel( a, b, d, c );
    //floorplan.csys.add_constraint_angle( a, b, c, None );

    // TODO: split these systems into Plugins for tidyness
    App::new()
        //.insert_resource(WinitSettings::desktop_app())
        .insert_resource( floorplan )
        .insert_resource(ClearColor(Color::srgb(0.176, 0.247, 0.431)))
        .init_resource::<floorplan::InteractionState>()
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin::default())
        .add_plugins(EguiPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(bevy_pancam::PanCamPlugin)
        .init_resource::<ui::OccupiedScreenSpace>()
        .add_systems(Startup, setup_system)
        .add_systems(Update, ui::ui_example_system)
        .add_systems(Update, diagram::render_diagram)
        .add_systems( Update, update_constraints )
        .add_systems( Update, interaction::cursor_events )
        .add_systems( Update, interaction::mouse_button_events )
        //.add_systems(Update, update_camera_transform_system)
        .run();
}


fn setup_system(
    mut commands: Commands,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
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



    // Spawn 3D scene
    /*
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(5.0, 5.0)),
        material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
        ..Default::default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500000.0,
            shadows_enabled: true,
            ..Default::default()
        },
         transform: Transform::from_xyz(4.0, 8.0, 4.0),
         ..Default::default()
    });
*/


    let camera_pos = Vec3::new(-2.0, 2.5, 5.0);
    //let camera_pos = Vec3::new(0.0, 100.0, 0.0);
    let camera_transform =
        Transform::from_translation(camera_pos).looking_at(CAMERA_TARGET, Vec3::Y);
    commands.insert_resource(OriginalCameraTransform(camera_transform));

    //commands.spawn((Camera3d::default(), camera_transform));
    commands.spawn(Camera3dBundle {
        camera: Camera {
            //clear_color: ClearColorConfig::None,
            order: 0,
            ..default()
        },
        transform: camera_transform,
        ..Default::default()
    });


}

/*
fn update_camera_transform_system(
    occupied_screen_space: Res<ui::OccupiedScreenSpace>,
    original_camera_transform: Res<OriginalCameraTransform>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&Projection, &mut Transform)>,
) {
    let (camera_projection, mut transform) = match camera_query.get_single_mut() {
        Ok((Projection::Perspective(projection), transform)) => (projection, transform),
        _ => unreachable!(),
    };

    let distance_to_target = (CAMERA_TARGET - original_camera_transform.translation).length();
    let frustum_height = 2.0 * distance_to_target * (camera_projection.fov * 0.5).tan();
    let frustum_width = frustum_height * camera_projection.aspect_ratio;

    let window = windows.single();

    let left_taken = occupied_screen_space.left / window.width();
    let right_taken = occupied_screen_space.right / window.width();
    let top_taken = occupied_screen_space.top / window.height();
    let bottom_taken = occupied_screen_space.bottom / window.height();
    transform.translation = original_camera_transform.translation
        + transform.rotation.mul_vec3(Vec3::new(
            (right_taken - left_taken) * frustum_width * 0.5,
            (top_taken - bottom_taken) * frustum_height * 0.5,
            0.0,
        ));
}
*/

fn update_constraints( mut floorplan : ResMut<floorplan::Floorplan> )
{
    // update the constraint solver
    floorplan.csys.eval_system();
}



