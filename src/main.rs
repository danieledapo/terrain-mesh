use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};

use simdnoise::NoiseBuilder;

#[derive(Debug, Clone)]
pub struct Terrain {
    heights: Vec<f32>,
    width: usize,
    height: usize,
}

impl Terrain {
    pub fn generate(width: usize, height: usize, amplitude: f32) -> Self {
        let gen = NoiseBuilder::fbm_2d(width, height);
        let heights = gen.generate_scaled(0.0, amplitude);

        Terrain {
            heights,
            width,
            height,
        }
    }

    pub fn z_at(&self, x: usize, y: usize) -> f32 {
        self.heights[y * self.width + x]
    }

    pub fn iter_by_row(&self) -> impl Iterator<Item = (usize, usize, f32)> + '_ {
        self.heights
            .iter()
            .enumerate()
            .map(move |(i, z)| (i / self.width, i % self.width, *z))
    }

    pub fn iter_pos_by_row(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..self.height).flat_map(move |y| (0..self.width).map(move |x| (y, x)))
    }

    pub fn index_of(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

fn main() -> io::Result<()> {
    let grid_size = 200;
    let amplitude = 20.0;

    let terrain = Terrain::generate(grid_size, grid_size, amplitude);

    let f = File::create("terrain.obj")?;
    let mut f = BufWriter::new(f);
    dump(&mut f, &terrain, true)
}

pub fn dump(w: &mut impl Write, terrain: &Terrain, support: bool) -> io::Result<()> {
    writeln!(w, "o terrain")?;

    for (y, x, z) in terrain.iter_by_row() {
        writeln!(w, "v {} {} {}", x, y, z)?;
    }

    if support {
        for (y, x) in terrain.iter_pos_by_row() {
            writeln!(w, "v {} {} 0", x, y)?;
        }
    }

    let height = terrain.height();
    let width = terrain.width();
    for y in 0..height.saturating_sub(1) {
        for x in 0..width.saturating_sub(1) {
            let i = 1 + terrain.index_of(x, y);
            let j = 1 + terrain.index_of(x, y + 1);
            writeln!(w, "f {} {} {} {}", i, j, j + 1, i + 1)?;
        }
    }

    if support {
        let oi = width * height + 1;
        writeln!(
            w,
            "f {} {} {} {}",
            oi,
            oi + terrain.index_of(0, height - 1),
            oi + terrain.index_of(width - 1, height - 1),
            oi + terrain.index_of(width - 1, 0),
        )?;

        for y in 0..height.saturating_sub(1) {
            writeln!(
                w,
                "f {} {} {} {}",
                oi + terrain.index_of(0, y + 1),
                oi + terrain.index_of(0, y),
                1 + terrain.index_of(0, y),
                1 + terrain.index_of(0, y + 1),
            )?;

            writeln!(
                w,
                "f {} {} {} {}",
                oi + terrain.index_of(width - 1, y),
                oi + terrain.index_of(width - 1, y + 1),
                1 + terrain.index_of(width - 1, y + 1),
                1 + terrain.index_of(width - 1, y),
            )?;
        }

        for x in 0..width.saturating_sub(1) {
            writeln!(
                w,
                "f {} {} {} {}",
                oi + terrain.index_of(x, 0),
                oi + terrain.index_of(x + 1, 0),
                1 + terrain.index_of(x + 1, 0),
                1 + terrain.index_of(x, 0),
            )?;

            writeln!(
                w,
                "f {} {} {} {}",
                oi + terrain.index_of(x + 1, height - 1),
                oi + terrain.index_of(x, height - 1),
                1 + terrain.index_of(x, height - 1),
                1 + terrain.index_of(x + 1, height - 1),
            )?;
        }
    }

    Ok(())
}
