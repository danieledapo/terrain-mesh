use crate::geo::{Bbox, Vec2};

#[derive(Debug)]
pub struct Bvh<Elem> {
    pub root: BvhNode<Elem>,
}

#[derive(Debug)]
pub enum BvhNode<Elem> {
    Leaf {
        elems: Vec<(Elem, Vec2)>,
        bbox: Bbox,
    },
    Branch {
        bbox: Bbox,
        children: Box<[BvhNode<Elem>; 4]>,
    },
}

impl<Elem> Bvh<Elem> {
    pub fn new(bbox: Bbox) -> Self {
        Bvh {
            root: BvhNode::Leaf {
                elems: Vec::with_capacity(64),
                bbox,
            },
        }
    }

    pub fn insert(&mut self, e: Elem, refpoint: Vec2) {
        self.root.insert(e, refpoint);
    }

    pub fn enclosing(
        &self,
        refpoint: Vec2,
        contains: impl Fn(&Elem, Vec2) -> bool,
    ) -> impl Iterator<Item = &Elem> {
        self.root.enclosing(refpoint, contains)
    }
}

impl<Elem> BvhNode<Elem> {
    pub fn insert(&mut self, e: Elem, refpoint: Vec2) {
        match self {
            BvhNode::Leaf { elems, bbox } => {
                elems.push((e, refpoint));
                bbox.expand(refpoint);

                if elems.len() > 64 {
                    let pivot = bbox.center();
                    let quads = bbox.split(pivot);

                    let mut children = Box::new([
                        BvhNode::Leaf {
                            elems: Vec::with_capacity(64),
                            bbox: quads[0],
                        },
                        BvhNode::Leaf {
                            elems: Vec::with_capacity(64),
                            bbox: quads[1],
                        },
                        BvhNode::Leaf {
                            elems: Vec::with_capacity(64),
                            bbox: quads[2],
                        },
                        BvhNode::Leaf {
                            elems: Vec::with_capacity(64),
                            bbox: quads[3],
                        },
                    ]);

                    for (e, refpoint) in elems.drain(0..) {
                        for child in children.iter_mut() {
                            if child.contains(refpoint) {
                                child.insert(e, refpoint);
                                break;
                            }
                        }
                    }

                    *self = BvhNode::Branch {
                        children,
                        bbox: *bbox,
                    };
                }
            }
            BvhNode::Branch { children, .. } => {
                for child in children.iter_mut() {
                    if child.contains(refpoint) {
                        child.insert(e, refpoint);
                        break;
                    }
                }
            }
        }
    }

    pub fn enclosing(
        &self,
        refpoint: Vec2,
        contains: impl Fn(&Elem, Vec2) -> bool,
    ) -> impl Iterator<Item = &Elem> {
        let mut nodes = vec![self];
        let mut cur_elems = [].iter();

        std::iter::from_fn(move || loop {
            for (e, _) in cur_elems.by_ref() {
                if contains(e, refpoint) {
                    return Some(e);
                }
            }

            let n = nodes.pop()?;
            match n {
                BvhNode::Leaf { elems, .. } => cur_elems = elems.iter(),
                BvhNode::Branch { children, bbox } => {
                    if bbox.contains(refpoint) {
                        nodes.extend(children.iter());
                    }
                }
            }
        })
    }

    pub fn contains(&self, p: Vec2) -> bool {
        let bbox = match self {
            BvhNode::Branch { bbox, .. } | BvhNode::Leaf { bbox, .. } => bbox,
        };

        bbox.contains(p)
    }
}
