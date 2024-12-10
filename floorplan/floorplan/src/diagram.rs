use bevy::{prelude::* };
use bevy_vello::{ prelude::* };

use constraints::{ Constraint, PinMode };

use vello::peniko::Color;

use super::floorplan::{Floorplan, WallStyle};
use super::interaction::{InteractionMode, InteractionState};

// Good talk about Vello:
//"Vello: High Performance 2d graphics - Raph Levien"
//https://www.youtube.com/watch?v=mmW_RbTyj8c

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
                    floorplan: Res<Floorplan>,
                    state: Res<InteractionState>,
                    ) {

    let (mut _transform, mut scene) = query_scene.single_mut();

    // Reset scene every frame
    *scene = VelloScene::default();

    if state.mode == InteractionMode::Preview {
        return;
    }

    let c_walls = Color::rgba8( 109, 123, 166, 255 );
    let c_constraint = Color::rgba8(188, 175, 171, 255 );
    let c_select =Color::rgba8( 252, 194, 225, 255 );
    let c_ghost = Color::rgba8( 76, 73, 166, 255);

    // If align mode (holding shift), draw the align line
    if state.do_align_cursor {
        let stroke = kurbo::Stroke::new(1.0).with_dashes( 0.0, [ 1.0, 4.0 ]);
        let cursor_diff = (state.world_cursor - state.world_cursor_align).abs();
        let align_p = state.world_cursor_align.diagp();
        let align_guide = if cursor_diff.x > cursor_diff.y {
            kurbo::Line::new(  kurbo::Point::new( align_p.x - 1000.0, align_p.y ),
                                kurbo::Point::new( align_p.x + 1000.0, align_p.y ) )
        } else {
            kurbo::Line::new(  kurbo::Point::new( align_p.x, align_p.y - 1000.0 ),
                                kurbo::Point::new( align_p.x, align_p.y + 1000.0 ) )
        };
        scene.stroke(&stroke, kurbo::Affine::IDENTITY, c_ghost, None, &align_guide);
    }

    // draw walls
    let stroke_pin = kurbo::Stroke::new(2.0);
    let stroke_ext = kurbo::Stroke::new(5.0);
    for (ndx, wall) in floorplan.walls.iter().enumerate() {

        let anc_a = floorplan.csys.anchors[ wall.anchor_a ];
        let anc_b = floorplan.csys.anchors[ wall.anchor_b ];
        let line = kurbo::Line::new( anc_a.p.diagp(), anc_b.p.diagp() );

        // match wall.style to pick stroke_int or stroke_ext
        // let stroke = match wall.style {
        //     WallStyle::Interior => &stroke_int,
        //     WallStyle::Exterior => &stroke_ext,
        // };

        let wall_col = if state.selected_walls.contains( &ndx ) {
            c_select
        } else {
            c_walls
        };

        scene.stroke(&stroke_ext, kurbo::Affine::IDENTITY, wall_col, None, &line);
    }


    // Draw anchors
    for (ndx, anc) in floorplan.csys.anchors.iter().enumerate() {

        let radius = match state.hover_anchor {
            Some(hover_ndx) if hover_ndx == ndx => 8.0,
            _ => 5.0,
        };

        let acolor = if state.selected_anchors.contains( &ndx ) {
            c_select
        } else {
            c_walls
        };

        // Draw crosshairs for pinned anchors
        if anc.pin == PinMode::PinY || anc.pin == PinMode::PinXY {
            let pin_offs_x = Vec2::new( 10.0, 0.0);
            let line = kurbo::Line::new(
                (anc.p - pin_offs_x).diagp(),
                (anc.p + pin_offs_x).diagp() );

            scene.stroke(&stroke_pin, kurbo::Affine::IDENTITY, acolor, None, &line);
        }

        if anc.pin == PinMode::PinX || anc.pin == PinMode::PinXY {
            let pin_offs_y = Vec2::new( 0.0, 10.0);
            let line = kurbo::Line::new(
                (anc.p - pin_offs_y).diagp(),
                (anc.p + pin_offs_y).diagp() );

            scene.stroke(&stroke_pin, kurbo::Affine::IDENTITY, acolor, None, &line);
        }

        scene.fill(
            peniko::Fill::NonZero,
            kurbo::Affine::default(),
            acolor,
            None,
            &kurbo::Circle::new( anc.p.diagp(), radius ),
        );
    }

    // Draw constraints
    let stroke_cons = kurbo::Stroke::new(2.5);
    let stroke_cons_dashed = kurbo::Stroke::new(2.0).with_dashes( 0.0, [ 2.0, 5.0 ]);
    for cons in floorplan.csys.constraints.iter() {

        match cons {
            Constraint::FixedLength( fixed_len ) => {

                let pa = floorplan.csys.anchors[ fixed_len.anc_a ].p;
                let pb = floorplan.csys.anchors[ fixed_len.anc_b ].p;

                match floorplan.find_wall( fixed_len.anc_a, fixed_len.anc_b ) {
                    Some( _wall) => {

                        let line = kurbo::Line::new( pa.diagp(), pb.diagp() );
                        scene.stroke(&stroke_cons, kurbo::Affine::IDENTITY,
                            c_constraint, None, &line);

                    }
                    None => {
                        let line = kurbo::Line::new( pa.diagp(), pb.diagp() );
                        scene.stroke(&stroke_cons_dashed, kurbo::Affine::IDENTITY,
                            c_constraint, None, &line);
                    }
                }
            }

            Constraint::Parallel( parallel ) => {

                let pa = floorplan.csys.anchors[ parallel.anc_a ].p;
                let pb = floorplan.csys.anchors[ parallel.anc_b ].p;
                let pc = floorplan.csys.anchors[ parallel.anc_c ].p;
                let pd = floorplan.csys.anchors[ parallel.anc_d ].p;

                draw_constraint_parr( &mut scene, stroke_cons.clone(), c_constraint, pa, pb );
                draw_constraint_parr( &mut scene, stroke_cons.clone(), c_constraint, pc, pd );

            }

            Constraint::Angle( angle ) => {

                let pa = floorplan.csys.anchors[ angle.anc_a ].p;
                let pb = floorplan.csys.anchors[ angle.anc_b ].p;
                let pc = floorplan.csys.anchors[ angle.anc_c ].p;

                let ba = (pa - pb).normalize() * 15.0;
                let bc = (pc - pb).normalize() * 15.0;

                let p2 = pb + ba + bc;
                let line = kurbo::Line::new( (pb + ba).diagp(), p2.diagp() );
                scene.stroke(&stroke_cons, kurbo::Affine::IDENTITY,
                            c_constraint, None, &line);

                let line = kurbo::Line::new( (pb + bc).diagp(), p2.diagp() );
                scene.stroke(&stroke_cons, kurbo::Affine::IDENTITY,
                    c_constraint, None, &line);

            }
        }
    }

    // If we're dragging a new wall, draw the ghost cursor
    if state.mode==InteractionMode::Create && state.create.is_dragging {

        let start_pos = match state.create.anc_start {
            Some(anc_ndx) => floorplan.csys.anchors[ anc_ndx ].p,
            None => state.create.drag_start,
        };

        let end_pos = match state.create.anc_end {
            Some(anc_ndx) => floorplan.csys.anchors[ anc_ndx ].p,
            None => state.create.drag_end,
        };

        // println!("Create drag {:?} {:?}", state.create.anc_start, state.create.anc_end );

        let line = kurbo::Line::new( start_pos.diagp(), end_pos.diagp() );
            scene.stroke(&stroke_cons, kurbo::Affine::IDENTITY,
                        c_ghost, None, &line);
    }
}

fn draw_constraint_parr( scene : &mut VelloScene, stroke_cons : kurbo::Stroke, brush : peniko::Color, pa : Vec2, pb : Vec2 )
{
    let ctr = (pa + pb) * 0.5;
    let ab = (pb -pa).normalize();
    let perp = Vec2::new( ab.y, -ab.x ) * 5.0;

    let ab = ab * 5.0;

    let line = kurbo::Line::new( (ctr + ab + perp).diagp(), (ctr - ab + perp).diagp() );
    scene.stroke(&stroke_cons, kurbo::Affine::IDENTITY,
        brush, None, &line);

        let line = kurbo::Line::new( (ctr + ab - perp).diagp(), (ctr - ab - perp).diagp() );
        scene.stroke(&stroke_cons, kurbo::Affine::IDENTITY,
            brush, None, &line);
}
