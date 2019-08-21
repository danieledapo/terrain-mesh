use std::collections::HashSet;

use crate::arena::{Arena, ArenaId};
use crate::bvh::Bvh;
use crate::geo::{Bbox, Circle, Vec2};

#[derive(Debug)]
pub struct DelaunayMesh {
    triangles: Arena<Triangle>,
    vertices: Arena<Vertex>,
    triangles_index: Bvh<ArenaId<Triangle>>,
}

#[derive(Debug)]
pub struct Triangle {
    vertices: [ArenaId<Vertex>; 3],
    circumcircle: Circle,
}

#[derive(Debug)]
pub struct Vertex {
    position: Vec2,
}

/// Region of interest that contains all the new/modified triangles after having inserted a point.
#[derive(Debug)]
pub struct Roi {
    triangles: Vec<ArenaId<Triangle>>,
}

impl DelaunayMesh {
    pub fn new(mut bbox: Bbox) -> Self {
        let _input_bbox = bbox;

        // add a bit of padding to account for the super triangles and to avoid degenerate
        // triangles.
        bbox.expand(bbox.min() - 20.0);
        bbox.expand(bbox.max() + 20.0);

        let mut dm = DelaunayMesh {
            triangles: Arena::new(),
            vertices: Arena::new(),
            triangles_index: Bvh::new(bbox),
        };

        let mut add_super_triangle = |a, b, c| {
            let va = dm.vertices.push(Vertex::new(a));
            let vb = dm.vertices.push(Vertex::new(b));
            let vc = dm.vertices.push(Vertex::new(c));

            dm.insert_triangle(va, vb, vc);
        };

        let min = bbox.min();
        let max = bbox.max();
        add_super_triangle(min, max, Vec2::new(min.y, max.x));
        add_super_triangle(max, min, Vec2::new(min.x, max.y));

        dm
    }

    // pub fn triangles(&self) -> impl Iterator<Item = &Triangle> {
    //     // NOTE: exclude super triangles' children
    // }

    pub fn insert(&mut self, p: Vec2) -> Roi {
        //
        // The idea here is to first find all the triangles whose circumcircle contains the new
        // point.
        //
        // Such triangles are then removed from the triangulation and replaced by a new set of
        // triangles that are generated by connecting the edges of the boundary of the removed
        // triangles to the new point.
        //
        //
        // Example
        //
        // ⡟⢍⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⡇       ⡏⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⡇       ⡟⠫⡉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⡹⡇
        // ⡇⠀⠑⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇       ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇       ⡇⠀⠈⠑⠤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡔⠁⡇
        // ⡇⠀⠀⠀⠑⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇       ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇       ⡇⠀⠀⠀⠀⠈⠒⢄⡀⠀⠀⠀⠀⠀⠀⠀⢀⠜⠀⠀⡇
        // ⡇⠀⠀⠀⠀⠀⠑⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇       ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇       ⡇⠀⠀⠀⠀⠀⠀⠀⠈⠢⢄⠀⠀⠀⠀⢀⠎⠀⠀⠀⡇
        // ⡇⠀⠀⠀⠀⠀⠀⠀⠑⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇       ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇       ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠢⣀⢠⠃⠀⠀⠀⠀⡇
        // ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠑⢄⠀⠀⠁⠀⠀⠀⠀⠀⡇  ==>  ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠁⠀⠀⠀⠀⠀⡇  ==>  ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡠⠒⠱⡀⠀⠀⠀⠀⡇
        // ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠑⢄⠀⠀⠀⠀⠀⠀⡇       ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇       ⡇⠀⠀⠀⠀⠀⠀⠀⠀⡠⠔⠉⠀⠀⠀⠑⡄⠀⠀⠀⡇
        // ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠑⢄⠀⠀⠀⠀⡇       ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇       ⡇⠀⠀⠀⠀⠀⣀⠔⠊⠀⠀⠀⠀⠀⠀⠀⠘⢄⠀⠀⡇
        // ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠑⢄⠀⠀⡇       ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇       ⡇⠀⠀⢀⠤⠊⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢆⠀⡇
        // ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠑⢄⡇       ⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇       ⣇⡠⠊⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢣⡇
        // ⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠁       ⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠁       ⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠁
        // current triangulation       remove intersecting         re-triangulate the boundary
        //                             triangles and find the      connecting its edges to
        //                             boundary of those           the new point
        //                             triangles
        //
        // Note that the circumcircles are not drawn for simplicity, just assume the point lies
        // inside the circumcircles of both triangles.
        //

        let bad_tris = self
            .triangles_index
            .enclosing(p, |&tid, p| self.triangles[tid].circumcircle.contains(p))
            .cloned()
            .collect::<Vec<_>>();

        // the boundary of the roi is the set of the outer edges that are not shared among the
        // enclosing triangles
        let mut boundary = HashSet::new();
        for tri in &bad_tris {
            let tri = &self.triangles[*tri];

            for v in 0..tri.vertices.len() {
                let edge = (tri.vertices[v], tri.vertices[(v + 1) % tri.vertices.len()]);

                if !boundary.insert(edge) {
                    boundary.remove(&edge);
                }

                let edge = (edge.1, edge.0);
                if !boundary.insert(edge) {
                    boundary.remove(&edge);
                }
            }
        }

        for tri in bad_tris {
            self.triangles.remove(tri);
        }

        let vp = self.vertices.push(Vertex::new(p));

        let mut roi = Vec::with_capacity(boundary.len());
        for (v0, v1) in boundary {
            let tri = self.insert_triangle(v0, v1, vp);
            roi.push(tri);
        }

        Roi { triangles: roi }
    }

    pub fn insert_triangle(
        &mut self,
        va: ArenaId<Vertex>,
        vb: ArenaId<Vertex>,
        vc: ArenaId<Vertex>,
    ) -> ArenaId<Triangle> {
        let a = self.vertices[va].position;
        let b = self.vertices[vb].position;
        let c = self.vertices[vc].position;

        let circumcircle = Circle::circumcircle(a, b, c);
        let tri = self.triangles.push(Triangle {
            vertices: [va, vb, vc],
            circumcircle,
        });

        self.triangles_index.insert(tri, circumcircle.center);
        tri
    }
}

impl Vertex {
    pub fn new(position: Vec2) -> Self {
        Vertex { position }
    }
}
