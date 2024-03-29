use std::env;
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::prelude::*;
use rand_pcg::Pcg32;

use clap::{Parser, Subcommand};
use simdnoise::NoiseBuilder;

/// Generate a terrain mesh from a noise function or a heightmap. The final mesh should be ready to
/// be 3d printed.
#[derive(Parser)]
pub struct App {
    /// Output obj filename template.
    #[arg(short, long, default_value = "terrain.obj")]
    output: PathBuf,

    /// Generate the dual of terrain too.
    #[arg(long)]
    dual: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Generate random terrain-like quad mesh using various types of noise functions.
    Random(RandomConfig),

    /// Turn grayscale 8 bit heightmap into a mesh.
    Heightmap(HeightmapConfig),
}

#[derive(Parser)]
pub struct RandomConfig {
    /// The width of the final terrain as in number of vertices.
    #[arg(short, long, default_value = "51")]
    width: u16,

    /// The depth of the final terrain as in number of vertices.
    #[arg(short, long, default_value = "51")]
    depth: u16,

    /// The seed to use to generate the terrain. You can find the seed of a given terrain by
    /// inspecting the obj file.
    #[arg(short, long)]
    seed: Option<u64>,

    #[arg(long, default_value = "0.5")]
    lacunarity: f32,

    #[arg(long, default_value = "4")]
    octaves: u8,

    #[arg(long, default_value = "2.0")]
    gain: f32,

    #[arg(long, default_value = "0.2")]
    frequency: f32,

    /// The maximum height of the terrain. If `base-thickness` is specified then the final mesh has
    /// a potential maximum height of `base-thickness` + `amplitude`.
    #[arg(short, long, default_value = "20")]
    amplitude: f32,

    /// The thickness of the base upon which the terrain is generated.
    #[arg(long = "base-thickness", default_value = "0.0")]
    base_thickness: f32,
}

#[derive(Parser)]
pub struct HeightmapConfig {
    /// Input grayscale heightmap.
    #[arg()]
    grayscale_heightmap: PathBuf,

    /// The maximum height of the terrain. If `base-thickness` is specified then the final mesh has
    /// a potential maximum height of `base-thickness` + `amplitude`.
    #[arg(short, long, default_value = "20")]
    amplitude: f32,

    /// The thickness of the base upon which the terrain is generated.
    #[arg(long = "base-thickness", default_value = "0.0")]
    base_thickness: f32,

    /// How much to smooth the grayscale image before turning it into a mesh. Smoothing is
    /// performed via a Gaussian blur.
    #[arg(short, long, default_value = "0.3")]
    smoothness: f32,
}

#[derive(Debug)]
pub struct Terrain {
    heights: Vec<f32>,
    width: usize,
    depth: usize,
    amplitude: f32,
    generator: TerrainGenerator,
}

#[derive(Debug, Clone)]
pub enum TerrainGenerator {
    Noise { seed: u64 },
    Dual { parent_seed: u64 },
    Heightmap,
}

impl Terrain {
    pub fn generate(
        RandomConfig {
            amplitude,
            base_thickness,
            depth,
            frequency,
            gain,
            lacunarity,
            octaves,
            seed,
            width,
        }: &RandomConfig,
    ) -> Self {
        let seed = seed.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time drift detected, aborting")
                .as_secs()
        });
        let noise_seed = Pcg32::seed_from_u64(seed).gen::<i32>();

        let width = usize::from(*width);
        let depth = usize::from(*depth);

        let mut noise_config = NoiseBuilder::fbm_2d(width, depth);
        noise_config
            .with_octaves(*octaves)
            .with_freq(*frequency)
            .with_gain(*gain)
            .with_lacunarity(*lacunarity)
            .with_seed(noise_seed);

        let heights = noise_config.generate_scaled(*base_thickness, base_thickness + *amplitude);

        Terrain {
            depth,
            heights,
            width,
            amplitude: *amplitude,
            generator: TerrainGenerator::Noise { seed },
        }
    }

    pub fn from_heightmap(
        HeightmapConfig {
            amplitude,
            base_thickness,
            grayscale_heightmap,
            smoothness,
        }: &HeightmapConfig,
    ) -> image::ImageResult<Self> {
        use std::convert::TryFrom;

        let img = image::open(grayscale_heightmap)?.to_luma8();
        let img = image::imageops::blur(&img, *smoothness);

        let (width, depth) = img.dimensions();
        let width = usize::try_from(width).unwrap();
        let depth = usize::try_from(depth).unwrap();

        let mut heights = vec![0.0; depth * width];
        for (x, y, p) in img.enumerate_pixels() {
            let x = usize::try_from(x).unwrap();
            let y = usize::try_from(y).unwrap();
            let i = (depth - 1 - y) * width + x;

            heights[i] = base_thickness + f32::from(p.0[0]) / 255.0 * amplitude;
        }

        Ok(Terrain {
            depth,
            heights,
            width,
            amplitude: *amplitude,
            generator: TerrainGenerator::Heightmap,
        })
    }

    pub fn dual(&self) -> Terrain {
        let heights = self
            .positions_by_depth()
            .map(|(y, x)| self.amplitude - self.height_at(self.width - 1 - x, y))
            .collect::<Vec<_>>();

        let generator = match self.generator {
            TerrainGenerator::Noise { seed } => TerrainGenerator::Dual { parent_seed: seed },
            TerrainGenerator::Dual { parent_seed } => TerrainGenerator::Noise { seed: parent_seed },
            TerrainGenerator::Heightmap => TerrainGenerator::Heightmap,
        };

        Terrain {
            heights,
            generator,
            ..*self
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

    pub fn amplitude(&self) -> f32 {
        self.amplitude
    }

    pub fn generator(&self) -> &TerrainGenerator {
        &self.generator
    }
}

fn main() -> image::ImageResult<()> {
    let opt = App::parse();

    let terrain = match opt.command {
        Command::Random(cfg) => Terrain::generate(&cfg),
        Command::Heightmap(cfg) => Terrain::from_heightmap(&cfg)?,
    };

    let mut f = BufWriter::new(File::create(&opt.output)?);
    dump(&mut f, &terrain, true)?;

    if opt.dual {
        let dual = terrain.dual();

        let dual_output = opt.output.with_file_name(format!(
            "{}-dual.{}",
            opt.output
                .file_stem()
                .map_or_else(|| "terrain".into(), |oss| oss.to_string_lossy()),
            opt.output
                .extension()
                .map_or_else(|| "obj".into(), |oss| oss.to_string_lossy()),
        ));

        let mut f = BufWriter::new(File::create(dual_output)?);
        dump(&mut f, &dual, true)?;
    }

    Ok(())
}

pub fn dump(w: &mut impl Write, terrain: &Terrain, support: bool) -> io::Result<()> {
    writeln!(
        w,
        r#"# generated by terrain-mesh <https://github.com/danieledapo/terrain-mesh>
# {}{}
o terrain"#,
        env::args().collect::<Vec<_>>().join(" "),
        match terrain.generator {
            TerrainGenerator::Noise { seed } | TerrainGenerator::Dual { parent_seed: seed } => {
                format!("\n# seed: {}", seed)
            }
            TerrainGenerator::Heightmap => String::new(),
        }
    )?;

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
            writeln!(w, "f {} {} {} {}", i, i + 1, j + 1, j)?;
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
