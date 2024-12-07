use bevy::{prelude::* };
use bevy_vello::{ prelude::* };

use super::floorplan;

// =============================================================
// convert from bevy's Vec2 to vello's Point.
// fixme: Not sure why we need to flip the y when drawing but we do
pub trait DiagramConvert {
    fn diagp( &self ) -> kurbo::Point;
}
impl DiagramConvert for Vec2 {
    fn diagp( &self ) -> kurbo::Point {
        kurbo::Point::new( self.x.into(),  (-self.y).into() )
    }
}

// =============================================================
pub fn render_diagram(mut query_scene: Query<(&mut Transform, &mut VelloScene)>,
                    floorplan: Res<floorplan::Floorplan>,
                    state: Res<floorplan::InteractionState>,
                    //time: Res<Time>
                    ) {
    //let sin_time = time.elapsed_seconds().sin().mul_add(0.5, 0.5);
    let (mut _transform, mut scene) = query_scene.single_mut();


    // Reset scene every frame
    *scene = VelloScene::default();

    // draw walls
    let stroke_int = kurbo::Stroke::new(2.0);
    let stroke_ext = kurbo::Stroke::new(6.0);
    for (ndx, wall) in floorplan.walls.iter().enumerate() {

        let anc_a = floorplan.csys.anchors[ wall.anchor_a ];
        let anc_b = floorplan.csys.anchors[ wall.anchor_b ];

        // scene.fill(
        //     peniko::Fill::NonZero,
        //     kurbo::Affine::default(),
        //     peniko::Color::WHITE,
        //     None,
        //     &kurbo::Line::new(

        //         kurbo::Point::new( anc_a.p.x.into(),  (-anc_a.p.y).into() ),
        //         kurbo::Point::new( anc_b.p.x.into(),  (-anc_b.p.y).into() )
        //     )
        // );

        // let line = kurbo::Line::new( kurbo::Point::new( anc_a.p.x.into(), (-anc_a.p.y).into() ),
        //                             kurbo::Point::new( anc_b.p.x.into(),  (-anc_b.p.y).into() ));

        let line = kurbo::Line::new( anc_a.p.diagp(), anc_b.p.diagp() );

        // match wall.style to pick stroke_int or stroke_ext
        let stroke = match wall.style {
            floorplan::WallStyle::Interior => &stroke_int,
            floorplan::WallStyle::Exterior => &stroke_ext,
        };

        //let line_stroke_color = peniko::Color::new([0.5373, 0.7059, 0.9804, 1.]);
        let wall_col = if state.selected_walls.contains( &ndx ) {
            peniko::Color::LIME_GREEN
        } else {
            peniko::Color::WHITE
        };

        scene.stroke(&stroke, kurbo::Affine::IDENTITY, wall_col, None, &line);
    }


    // Draw anchors
    for (ndx, anc) in floorplan.csys.anchors.iter().enumerate() {

        let radius = match state.hover_anchor {
            Some(hover_ndx) if hover_ndx == ndx => 8.0,
            _ => 5.0,
        };

        let acolor = if state.selected_anchors.contains( &ndx ) {
            peniko::Color::LIGHT_GREEN
        } else {
            peniko::Color::GOLDENROD
        };

        scene.fill(
            peniko::Fill::NonZero,
            kurbo::Affine::default(),
            acolor,
            None,
            &kurbo::Circle::new( anc.p.diagp(), radius ),
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
