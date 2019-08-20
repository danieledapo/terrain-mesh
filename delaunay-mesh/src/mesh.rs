use crate::arena::{Arena, ArenaId};
use crate::bvh::Bvh;
use crate::geo::{Bbox, Circle, Vec2};

#[derive(Debug)]
pub struct DelaunayMesh {
    pub triangles: Arena<Triangle>,
    pub vertices: Arena<Vertex>,
    pub triangles_index: Bvh<ArenaId<Triangle>>,
}

impl DelaunayMesh {
    pub fn new(bbox: Bbox) -> Self {
        //
        // TODO: insert super triangles
        //

        DelaunayMesh {
            triangles: Arena::new(),
            vertices: Arena::new(),
            triangles_index: Bvh::new(bbox),
        }
    }

    pub fn insert(&mut self, p: Vec2) {
        //
        // TODO: implement
        //

        let _tris = self
            .triangles_index
            .root
            .enclosing(p, |&tid, p| self.triangles[tid].circumcircle.contains(p));
    }
}

#[derive(Debug)]
pub struct Triangle {
    pub vertices: [ArenaId<Vertex>; 3],
    pub circumcircle: Circle,
}

#[derive(Debug)]
pub struct Vertex {
    pub position: Vec2,
    pub connected_faces: Vec<ArenaId<Triangle>>,
}
