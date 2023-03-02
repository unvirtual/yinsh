use itertools::Itertools;
use macroquad::models::Vertex;
use macroquad::prelude::*;
use std::{cmp::Eq, f32::consts::PI};

use crate::core::coord::{HexCoord, HexCoordF, Point};

pub fn build_grid_lines(radius: f32) -> Vec<[Point; 2]> {
    let dx: f32 = 0.5 * (3. as f32).sqrt();
    let mut res = Vec::new();

    // diagonals
    for dy in [-0.5 as f32, 0.5 as f32] {
        let lambda: f32 = radius / (1. - dy.powi(2)).sqrt();
        let (l0, l1) = ((-lambda).trunc() as i32, lambda.trunc() as i32);

        for l in l0..=l1 {
            let l = l as f32;
            let det = (l.powi(2) * (dy.powi(2) - 1.) + radius.powi(2)).sqrt();
            if det <= 0. {
                continue;
            }
            let mut mu1 = -l * dy - det;
            let mut mu2 = -l * dy + det;
            if l.abs() > radius {
                mu1 = mu1.ceil();
                mu2 = mu2.floor();
            } else {
                mu1 = mu1.trunc();
                mu2 = mu2.floor();
            }

            let vec = [Point(mu1 * dx, l + mu1 * dy), Point(mu2 * dx, l + mu2 * dy)];
            res.push(vec);
        }
    }

    // verticals
    let lambda: f32 = radius * 2. / 3. * (3. as f32).sqrt();
    let (l0, l1) = ((-lambda).trunc() as i32, lambda.trunc() as i32);

    for l in l0..=l1 {
        let l = l as f32;
        let det = (4. * radius.powi(2) - 3. * l.powi(2)).sqrt();
        if det <= 0. {
            continue;
        }
        let mut mu1 = 0.5 * (l - det);
        let mut mu2 = 0.5 * (l + det);
        if l.abs() > radius {
            mu1 = mu1.ceil();
            mu2 = mu2.floor();
        } else {
            mu1 = mu1.trunc();
            mu2 = mu2.floor();
        }

        let vec = [Point(l * dx, -0.5 * l + mu1), Point(l * dx, -0.5 * l + mu2)];
        res.push(vec);
    }
    res
}

pub fn build_grid_hull_mesh(line_endpoints: &Vec<[Point; 2]>) -> (Vec<Vertex>, Vec<u16>) {
    let mut line_endpoints = line_endpoints
        .iter()
        .flatten()
        .map(|pt| HexCoord::closest_coord_to_point(pt).0)
        .unique()
        .map(Point::from)
        .collect::<Vec<_>>();

    line_endpoints.sort_unstable_by(|pt1, pt2| {
        pt1.1
            .atan2(pt1.0)
            .partial_cmp(&pt2.1.atan2(pt2.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut vertices = vec![Vertex {
        position: vec3(0., 0., 0.),
        uv: vec2(0., 0.),
            color: Color::from_rgba(213, 240, 245, 255),
    }];
    let mut indices: Vec<u16> = vec![];

    for (i, pt) in line_endpoints.iter().enumerate() {
        vertices.push(Vertex {
            position: vec3(pt.0, pt.1, 0.),
            uv: vec2(0., 0.),
            color: Color::from_rgba(162, 222, 255, 255),
        });
        if i > 0 {
            let mut next = (i+1) % line_endpoints.len();
            if next == 0 {
                next += 1;
            }

            indices.push(0);
            indices.push(next as u16);
            indices.push(i as u16);
        }
    }

    (vertices, indices)
}

pub fn draw_ring_mesh(x: f32, y: f32, inner: f32, outer: f32, color: Color, segments: u16) {
    draw_arc_mesh(x, y, inner, outer, 2. * PI, vec2(1., 0.), color, segments);
}

pub fn draw_arc_mesh(
    x: f32,
    y: f32,
    inner: f32,
    outer: f32,
    angle: f32,
    startvec: Vec2,
    color: Color,
    segments: u16,
) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let n = segments;
    let delta = angle / n as f32;

    let dot = startvec.dot(vec2(1., 0.));
    let det = -startvec.y;
    let phase = det.atan2(dot);
    let mut alpha: f32 = -phase;

    vertices.push(Vertex {
        position: Vec3::new(x + inner * alpha.cos(), y + inner * alpha.sin(), 0.),
        uv: Vec2::new(0., 0.),
        color,
    });

    vertices.push(Vertex {
        position: Vec3::new(x + outer * alpha.cos(), y + outer * alpha.sin(), 0.),
        uv: Vec2::new(0., 0.),
        color,
    });

    for i in 0..n {
        alpha += delta;
        vertices.push(Vertex {
            position: Vec3::new(x + inner * alpha.cos(), y + inner * alpha.sin(), 0.),
            uv: Vec2::new(0., 0.),
            color,
        });
        vertices.push(Vertex {
            position: Vec3::new(x + outer * alpha.cos(), y + outer * alpha.sin(), 0.),
            uv: Vec2::new(0., 0.),
            color,
        });

        indices.push(0 + 2 * i);
        indices.push(1 + 2 * i);
        indices.push(3 + 2 * i);

        indices.push(0 + 2 * i);
        indices.push(3 + 2 * i);
        indices.push(2 + 2 * i);
    }

    let mesh = Mesh {
        vertices,
        indices,
        texture: None,
    };

    draw_mesh(&mesh);
}

fn build_circle_sector_mesh(
    pt: Vec2,
    radius: f32,
    startvec: Vec2,
    degrees: f32,
    startindex: u16,
    samples: u16,
    color: Color,
) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    vertices.push(Vertex {
        position: Vec3::from((pt, 0.)),
        uv: Vec2::new(0., 0.),
        color,
    });

    for i in 0..samples {
        let dot = startvec.dot(vec2(1., 0.));
        let det = -startvec.y;
        let angle = det.atan2(dot);
        let x = pt.x + radius * (-angle + i as f32 * degrees / (samples - 1) as f32).cos();
        let y = pt.y + radius * (-angle + i as f32 * degrees / (samples - 1) as f32).sin();
        vertices.push(Vertex {
            position: Vec3::new(x, y, 0.),
            uv: Vec2::new(0., 0.),
            color,
        });
        if i < samples - 1 {
            indices.push(startindex);
            indices.push(startindex + i + 1);
            indices.push(startindex + i + 2);
        }
    }
    (vertices, indices)
}

pub fn draw_line_mesh(start: Vec2, end: Vec2, height: f32, color: Color) {
    let dir = (end - start).normalize();
    let perp = Vec2::new(dir.y, -dir.x);

    let mut indices: Vec<u16> = vec![];

    let vertices = [
        start + 0.5 * height * perp,
        start - 0.5 * height * perp,
        end + 0.5 * height * perp,
        end - 0.5 * height * perp,
    ]
    .map(|pos2d| Vertex {
        position: Vec3::from((pos2d, 0.)),
        uv: vec2(0., 0.),
        color,
    })
    .to_vec();

    indices.push(0);
    indices.push(1);
    indices.push(2);

    indices.push(2);
    indices.push(1);
    indices.push(3);

    let mesh = Mesh {
        vertices,
        indices,
        texture: None,
    };

    draw_mesh(&mesh);
}

pub fn draw_run_indicator(
    corners: &[Vec2; 4],
    perp: &Vec2,
    height: f32,
    samples: u16,
    color: Color,
    line_color: Color,
) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    let left_circle_center = corners[0] - 0.5 * height * (*perp);
    let right_circle_center = corners[2] + 0.5 * height * (*perp);

    let left_circle = build_circle_sector_mesh(
        left_circle_center,
        0.5 * height,
        -1. * (*perp),
        PI,
        0,
        samples,
        color,
    );
    let right_circle = build_circle_sector_mesh(
        right_circle_center,
        0.5 * height,
        *perp,
        PI,
        samples + 1,
        samples,
        color,
    );

    vertices.extend(left_circle.0);
    indices.extend(left_circle.1);
    vertices.extend(right_circle.0);
    indices.extend(right_circle.1);

    indices.push(0);
    indices.push(samples + 1);
    indices.push(1);

    indices.push(1);
    indices.push(samples + 1);
    indices.push(2 * samples + 1);

    indices.push(0);
    indices.push(samples + 2);
    indices.push(samples + 1);

    indices.push(0);
    indices.push(samples);
    indices.push(samples + 2);

    let mesh = Mesh {
        vertices,
        indices,
        texture: None,
    };
    draw_mesh(&mesh);

    draw_line_mesh(corners[1], corners[2], 0.05, line_color);
    draw_line_mesh(corners[0], corners[3], 0.05, line_color);
    draw_arc_mesh(
        left_circle_center.x,
        left_circle_center.y,
        0.5 * height - 0.025,
        0.5 * height + 0.025,
        PI,
        -1. * (*perp),
        line_color,
        10,
    );
    draw_arc_mesh(
        right_circle_center.x,
        right_circle_center.y,
        0.5 * height - 0.025,
        0.5 * height + 0.025,
        PI,
        *perp,
        line_color,
        10,
    );
}
