use bevy::{prelude::* };

// This is super messy, just added it for fun, don't look
// too closely :) :)

const CAMERA_TARGET: Vec3 = Vec3::ZERO;

const PREVIEW_SCALE: f32 = 0.04;
const PREVIEW_TIME: f32 = 0.8;

// TODO: figure out how to get this from the gltf scene
const MESH_WIDTH: f32 = 2.0;

use super::floorplan;
use rand::Rng;

use super::interaction::{InteractionMode, InteractionState};

#[derive(Event)]
pub struct RebuildFloorplan;

#[derive(Component)]
pub struct PreviewGeo;

#[derive(Component)]
pub struct PreviewCamera {
    pub preview_radius : f32,
    pub lerptime : f32,
}

#[derive(Resource,Default)]
pub struct WallSet {
    pub walls : Vec<Handle<Mesh>>,
}

pub fn setup_preview (
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut walls: ResMut<WallSet>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // This is all rather hardcoded right now :P
    for i in 0..3 {
        // let mesh_handle = asset_server.load("walls.glb#Mesh0/Primitive0");
        let mesh_handle = asset_server.load(format!( "walls.glb#Mesh{}/Primitive0", i) );
        walls.walls.push( mesh_handle );
    }

    // Spawn 3D scene
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(100.0, 100.0)),
        material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
        ..Default::default()
    });
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
    //     material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
    //     transform: Transform::from_xyz(0.0, 0.5, 0.0),
    //     ..Default::default()
    // });
    // commands.spawn( SceneBundle {
    //     scene: walls_src.clone(),
    //     //material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
    //     // transform: Transform {
    //     //     translation: tile_origin,
    //     //     rotation: Quat::from_rotation_y( rot ),
    //     //     ..default()
    //     //   },
    //     ..default()
    // });

    commands.spawn(( PbrBundle {
        mesh: walls.walls[0].clone(),
        material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
     }, PreviewGeo ));

    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 1500000.0,
    //         shadows_enabled: true,
    //         ..Default::default()
    //     },
    //      transform: Transform::from_xyz(4.0, 8.0, 4.0),
    //      ..Default::default()
    // });
    commands.spawn( DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });



    let camera_pos = Vec3::new(-20.0, 25.0, 50.0);
    //let camera_pos = Vec3::new(0.0, 100.0, 0.0);
    let camera_transform =
        Transform::from_translation(camera_pos).looking_at(CAMERA_TARGET, Vec3::Y);

    //commands.spawn((Camera3d::default(), camera_transform));
    commands.spawn((Camera3dBundle {
        camera: Camera {
            //clear_color: ClearColorConfig::None,
            order: 0,
            ..default()
        },
        transform: camera_transform,
        ..Default::default()
    }, PreviewCamera { preview_radius : 5.0, lerptime : 0.0 } ) );
}


pub fn adjust_preview_camera(
    time: Res<Time>,
    mut camera_q: Query<(&mut Transform, &mut PreviewCamera)>,
    state : Res<InteractionState>,
) {
    let (mut transform, mut pcam) = camera_q.single_mut();


    pcam.lerptime = if state.mode == InteractionMode::Preview {
         (pcam.lerptime + time.delta_seconds()).min( PREVIEW_TIME )
    } else {
        (pcam.lerptime - time.delta_seconds()).max( 0.0 )
    };

    let t = time.elapsed_seconds() * 0.1;
    let dir = Vec3::new( t.cos(), 0.5, t.sin() );
    let cpos = dir * (pcam.preview_radius * 2.5);


    let lerpval = 1.0-(pcam.lerptime / PREVIEW_TIME);
    let yval = lerpval * 40.0;

    transform.translation = cpos.lerp( Vec3::new( 0.0, 4.0, 1.0 ), lerpval );
    transform.look_at( Vec3::new( 0.0, yval, 0.0 ), Vec3::Y );

}


pub fn rebuild_floorplan(
    walls: Res<WallSet>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ev_rebuild: EventReader<RebuildFloorplan>,
    despawn_q: Query<Entity, With<PreviewGeo>>,
    floorplan : Res<floorplan::Floorplan>,
    mut camera_q: Query<&mut PreviewCamera>,
) {
    for _ev in ev_rebuild.read() {
        println!("Need to rebuild floorplan...");

        // Despawn old preview geo
        for entity in &despawn_q {
            commands.entity(entity).despawn_recursive();
        }

        //let mesh_handle = asset_server.load("walls.glb#Mesh0/Primitive0");
        let mtl = materials.add(Color::srgb(0.8, 0.7, 0.6));

        let mut rng = rand::thread_rng();
        for wall in floorplan.walls.iter() {

            let pa = floorplan.csys.anchors[ wall.anchor_a ].p * PREVIEW_SCALE;
            let pb = floorplan.csys.anchors[ wall.anchor_b ].p * PREVIEW_SCALE;
            //let ctr = ((pa + pb) * 0.5) * PREVIEW_SCALE;

            let wall_len = pa.distance( pb );
            let num_segs = wall_len / MESH_WIDTH;
            let (num, stretch) = if num_segs < 1.0 {
                ( 1, wall_len / MESH_WIDTH )
            } else {
                let num_segs = num_segs.floor();
                let seg_w = wall_len / num_segs;
                ( num_segs as u32,  seg_w / MESH_WIDTH )
            };


            // don't make more then 10 segments for each wall
            let num = num.min( 10 );

            let dir = pb - pa;

            let dn = dir.normalize();
            let ang = -dn.y.atan2( dn.x );

            //println!("Num segments {}", num );

            let mut radius : f32 = 0.0;
            for i in 0..num {

                // messy random choice here, favor flat walls to "special" decorations
                let random_wall = rng.gen_range(0..walls.walls.len() + 5 );
                let random_index = if random_wall >= walls.walls.len() {
                    1 // todo: find a way to get this by name from the gltf instead of hardcoding it
                } else {
                    random_wall
                };

                let p = pa + (dir / num as f32) * (i as f32);
                commands.spawn(( PbrBundle {
                    mesh: walls.walls[ random_index ].clone(),
                    material: mtl.clone(),
                    //transform: Transform::from_xyz(p.x, 0.0, p.y),
                    transform: Transform {
                        translation : Vec3::new( p.x, 0.0, p.y ),
                        rotation: Quat::from_rotation_y( ang ),
                        scale: Vec3::new( stretch * 1.1, 1.0, 1.0 ),
                        ..default()
                    },
                    ..default()
                }, PreviewGeo ));

                radius = radius.max( p.length() );

                println!("Spawn {}/{} at {:?}", i, num, p );
            }

            let mut pcam = camera_q.single_mut();
            pcam.preview_radius = radius.max( 5.0 ).min( 20.0 );
            //println!("Radius is {} pcam {}", radius, pcam.preview_radius );
        }
    }
}
