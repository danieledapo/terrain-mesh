//!
//! Implementation of Fast Polygonal Approximation of terrain fields
//!

use crate::geo::{BarycentricCoords, Bbox, Vec2};

pub trait Heightfield {
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn height_at(&self, x: u32, y: u32) -> f64;
}

pub fn scape(heightfield: &impl Heightfield, max_vertices: usize) {
    let w = f64::from(heightfield.width()) - 1.0;
    let h = f64::from(heightfield.height()) - 1.0;

    let mut bbox = Bbox::new(Vec2::zero());
    bbox.expand(Vec2::new(w, h));

    let mut triangulation = DelaunayMesh::new(bbox);

    triangulation.insert(Vec2::zero());
    triangulation.insert(Vec2::new(w, h));
    triangulation.insert(Vec2::new(w, h));
    triangulation.insert(Vec2::new(w, h));

    // TODO: candidates should be a priority queue with the ability to remove an element
    let mut candidates = vec![];
    for (tri, _) in triangulation.triangles() {
        dbg!(tri);
        let vertices = triangulation.triangle_vertices(tri);

        let best_candidate = find_best_candidate(heightfield, vertices);
        if let Some((p, err)) = best_candidate {
            candidates.push((tri, p, err));
        }
    }

    for i in 0..max_vertices {
        candidates.sort_by(|(_, _, e1), (_, _, e2)| e2.partial_cmp(e1).unwrap());

        let (tri, p, err) = match candidates.pop() {
            None => break,
            Some(v) => v,
        };

        // TODO: the following insert does a spatial query to find the bounding triangle, but we
        // already know it, it's `tri`. Avoiding a spatial query might be a great speedup.
        let roi = triangulation.insert(p);

        candidates.retain(|(t, _, _)| roi.old_triangles.contains(t));

        for tri in roi.new_triangles {
            let vertices = triangulation.vertices(tri);

            let best_candidate = find_best_candidate(heightfield, vertices);
            if let Some((p, err)) = best_candidate {
                candidates.push((tri, p, err));
            }
        }
    }
}

fn find_best_candidate(heightfield: &impl Heightfield, vertices: [Vec2; 3]) -> Option<(Vec2, f64)> {
    //
    // The idea here is to find the pixel inside the triangle with the greatest difference between
    // the real value stored in the heightfield and the one calculated by interpolating it using
    // barycentric coordinates.
    //
    // This algorithm is basically a rasterization loop.
    //

    let mut bbox = Bbox::new(vertices[0]);
    bbox.expand(vertices[1]);
    bbox.expand(vertices[2]);

    let yrange = bbox.min().y as u32..=bbox.max().y as u32;
    let xrange = bbox.min().x as u32..=bbox.max().x as u32;

    yrange
        .flat_map(|y| xrange.clone().map(move |x| (x, y)))
        .filter_map(|(x, y)| {
            let p = Vec2::new(x.into(), y.into());
            let bary = BarycentricCoords::triangle(vertices, p)?;

            let v0 = (vertices[0].x as u32, vertices[0].y as u32);
            let v1 = (vertices[1].x as u32, vertices[1].y as u32);
            let v2 = (vertices[2].x as u32, vertices[2].y as u32);

            let real_h = heightfield.height_at(x, y);
            let interpolated_h = bary.interpolate([
                heightfield.height_at(v0.0, v0.1),
                heightfield.height_at(v1.0, v1.1),
                heightfield.height_at(v2.0, v2.1),
            ]);

            let err = real_h - interpolated_h;

            Some((p, err))
        })
        .max_by(|(_, err1), (_, err2)| err1.partial_cmp(err2).unwrap())
}
