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

use constraints::{ add, add_vec2, ConstraintSystem };

mod ui;


#[derive(Resource, Default)]
struct InteractionState {
    hover_anchor : Option<usize>,
    drag_anchor : Option<usize>,
}

const CAMERA_TARGET: Vec3 = Vec3::ZERO;

#[derive(Resource, Deref, DerefMut)]
struct OriginalCameraTransform(Transform);

#[derive(Resource)]
struct Floorplan
{
    csys : ConstraintSystem,
}


fn main() {

    let mut csys = ConstraintSystem::new();

    csys.add_anchor( Vec2::new( -100.0, -100.0 ));
    csys.add_anchor( Vec2::new(  100.0, -100.0 ));
    csys.add_anchor( Vec2::new( -100.0, 120.0 ));
    csys.add_anchor( Vec2::new(  100.0, 100.0 ));

    App::new()
        //.insert_resource(WinitSettings::desktop_app())
        .insert_resource( Floorplan { csys : csys })
        .insert_resource(ClearColor(Color::srgb(0.176, 0.247, 0.431)))
        .init_resource::<InteractionState>()
        .add_plugins(DefaultPlugins)
        .add_plugins(VelloPlugin::default())
        .add_plugins(EguiPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(bevy_pancam::PanCamPlugin)
        .init_resource::<ui::OccupiedScreenSpace>()
        .add_systems(Startup, setup_system)
        .add_systems(Update, ui::ui_example_system)
        .add_systems(Update, vello_animation)
        .add_systems( Update, cursor_events )
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

    let result = add_vec2( Vec2::new( 1.0,1.0 ), Vec2::new( 3.0, 5.0 ) );
    println!( "test add {:?} {:?}", add( 3, 4), result );

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

fn cursor_events(
    mut evr_cursor: EventReader<CursorMoved>,
    buttons: Res<ButtonInput<MouseButton>>,
    q_camera: Query<(&Camera, &Camera2d, &GlobalTransform)>,
    mut q_pancam : Query<&mut bevy_pancam::PanCam>,
    mut floorplan : ResMut<Floorplan>,
    mut state : ResMut<InteractionState>,
) {
    for ev in evr_cursor.read() {

        let ( cam, _, cam_transform ) = q_camera.single();

        let Some(world_pos) = cam.viewport_to_world_2d( cam_transform, ev.position ) else {
            return
        };

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

        state.drag_anchor = if buttons.pressed( MouseButton::Left ) {
             state.hover_anchor
        } else {


            None
        };

        // Adjust drag anchor
        if let Some(drag_anchor) = state.drag_anchor {
            floorplan.csys.anchors[drag_anchor].p = world_pos;
        }
    }
}


fn vello_animation(mut query_scene: Query<(&mut Transform, &mut VelloScene)>,
                    floorplan: Res<Floorplan>,
                    state: Res<InteractionState>,
                    //time: Res<Time>
                    ) {
    //let sin_time = time.elapsed_seconds().sin().mul_add(0.5, 0.5);
    let (mut _transform, mut scene) = query_scene.single_mut();


    // Reset scene every frame
    *scene = VelloScene::default();

    for (ndx, anc) in floorplan.csys.anchors.iter().enumerate() {

        let radius = match state.hover_anchor {
            Some(hover_ndx) if hover_ndx == ndx => 8.0,
            _ => 5.0,
        };

        scene.fill(
            peniko::Fill::NonZero,
            kurbo::Affine::default(),
            peniko::Color::GOLDENROD,
            None,
            &kurbo::Circle::new(
                 kurbo::Point::new( anc.p.x.into(),  (-anc.p.y).into() ), radius ),
        );

    }

    /*
    // Animate color green to blue
    let c = Vec3::lerp(
        Vec3::new(-1.0, 1.0, -1.0),
        Vec3::new(-1.0, 1.0, 1.0),
        sin_time + 0.5,
    );

    // Animate the corner radius
    scene.fill(
        peniko::Fill::NonZero,
        kurbo::Affine::default(),
        peniko::Color::rgb(c.x as f64, c.y as f64, c.z as f64),
        None,
        &kurbo::RoundedRect::new(-50.0, -50.0, 50.0, 50.0, (sin_time as f64) * 50.0),
    );

    scene.fill(
        peniko::Fill::NonZero,
        kurbo::Affine::default(),
        peniko::Color::rgb(1.0, 1.0, 0.2 ),
        None,
        &kurbo::RoundedRect::new(-20.0, -20.0, 20.0, 20.0, (sin_time as f64) * 20.0),
    );
    */

    // transform.scale = Vec3::lerp(Vec3::ONE * 0.5, Vec3::ONE * 1.0, sin_time);
    // transform.translation = Vec3::lerp(Vec3::X * -100.0, Vec3::X * 100.0, sin_time);
    // transform.rotation = Quat::from_rotation_z(-std::f32::consts::TAU * sin_time);
}

