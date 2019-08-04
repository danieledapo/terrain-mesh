use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use simdnoise::NoiseBuilder;
use structopt::StructOpt;

/// Generate random terrain-like meshes using various types of noise functions.
#[derive(StructOpt)]
pub struct App {
    /// Output obj file.
    #[structopt(short, long, parse(from_os_str), default_value = "terrain.obj")]
    output: PathBuf,

    /// The width of the final terrain.
    #[structopt(short, long, default_value = "200")]
    width: usize,

    /// The depth of the final terrain.
    #[structopt(short, long, default_value = "200")]
    depth: usize,

    /// The maximum height of the terrain.
    #[structopt(short, long, default_value = "20")]
    amplitude: f32,
}

#[derive(Debug, Clone)]
pub struct Terrain {
    heights: Vec<f32>,
    width: usize,
    depth: usize,
}

impl Terrain {
    pub fn generate(width: usize, depth: usize, amplitude: f32) -> Self {
        let mut gen = NoiseBuilder::fbm_2d(width, depth);
        gen.with_octaves(4).with_freq(0.2);

        let heights = gen.generate_scaled(0.0, amplitude);

        Terrain {
            heights,
            width,
            depth,
        }
    }

    pub fn height_at(&self, x: usize, y: usize) -> f32 {
        self.heights[y * self.width + x]
    }

    pub fn iter_by_depth(&self) -> impl Iterator<Item = (usize, usize, f32)> + '_ {
        self.heights
            .iter()
            .enumerate()
            .map(move |(i, z)| (i / self.width, i % self.width, *z))
    }

    pub fn positions_by_depth(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..self.depth).flat_map(move |y| (0..self.width).map(move |x| (y, x)))
    }

    pub fn index_of(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn depth(&self) -> usize {
        self.depth
    }
}

fn main() -> io::Result<()> {
    let opt = App::from_args();

    let terrain = Terrain::generate(opt.width, opt.depth, opt.amplitude);

    let f = File::create(opt.output)?;
    let mut f = BufWriter::new(f);
    dump(&mut f, &terrain, true)
}

pub fn dump(w: &mut impl Write, terrain: &Terrain, support: bool) -> io::Result<()> {
    writeln!(w, "o terrain")?;

    for (y, x, z) in terrain.iter_by_depth() {
        writeln!(w, "v {} {} {}", x, y, z)?;
    }

    if support {
        for (y, x) in terrain.positions_by_depth() {
            writeln!(w, "v {} {} 0", x, y)?;
        }
    }

    let depth = terrain.depth();
    let width = terrain.width();
    for y in 0..depth.saturating_sub(1) {
        for x in 0..width.saturating_sub(1) {
            let i = 1 + terrain.index_of(x, y);
            let j = 1 + terrain.index_of(x, y + 1);
            writeln!(w, "f {} {} {} {}", i, j, j + 1, i + 1)?;
        }
    }

    if support {
        let oi = width * depth + 1;
        writeln!(
            w,
            "f {} {} {} {}",
            oi,
            oi + terrain.index_of(0, depth - 1),
            oi + terrain.index_of(width - 1, depth - 1),
            oi + terrain.index_of(width - 1, 0),
        )?;

        for y in 0..depth.saturating_sub(1) {
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
                oi + terrain.index_of(x + 1, depth - 1),
                oi + terrain.index_of(x, depth - 1),
                1 + terrain.index_of(x, depth - 1),
                1 + terrain.index_of(x + 1, depth - 1),
            )?;
        }
    }

    Ok(())
}
