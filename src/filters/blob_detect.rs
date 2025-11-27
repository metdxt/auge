use crate::filters::{AugeFilter, FilterResult};
use crate::types::AugeError;
use clap::ValueEnum;
use image::{DynamicImage, GenericImageView, Pixel, Rgb, RgbImage, Rgba, RgbaImage};
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, Clone)]
pub struct Blob {
    pub points: Vec<(u32, u32)>,
    pub size: usize,
}

impl PartialEq for Blob {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size
    }
}

impl Eq for Blob {}

impl PartialOrd for Blob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Blob {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size.cmp(&other.size)
    }
}

impl Blob {
    pub fn new(points: Vec<(u32, u32)>) -> Self {
        let size = points.len();
        Self { points, size }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum BlobColorMode {
    Rainbow,
    Heatmap,
    Solid,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum BlobBackground {
    Black,
    Transparent,
    Original,
}

pub struct BlobDetectFilter {
    pub threshold: u8,
    pub target_color: Option<Rgb<u8>>,
    pub mode: BlobColorMode,
    pub background: BlobBackground,
}

impl AugeFilter for BlobDetectFilter {
    fn apply(&self, img: DynamicImage) -> Result<FilterResult, AugeError> {
        let width = img.width();
        let height = img.height();

        if width == 0 || height == 0 {
            return Ok(FilterResult::Image(DynamicImage::new_rgba8(0, 0)));
        }

        let blobs = find_blobs_tiled(&img, self.threshold, self.target_color);
        let max_blob_size = blobs.peek().map(|b| b.size).unwrap_or(0);

        let output_image = match self.background {
            BlobBackground::Black => {
                let mut canvas = RgbImage::new(width, height);
                draw_blobs(&mut canvas, blobs, &self.mode, max_blob_size);
                DynamicImage::ImageRgb8(canvas)
            }
            BlobBackground::Transparent => {
                let mut canvas = RgbaImage::new(width, height);
                draw_blobs(&mut canvas, blobs, &self.mode, max_blob_size);
                DynamicImage::ImageRgba8(canvas)
            }
            BlobBackground::Original => {
                let mut canvas = img.to_rgba8();
                draw_blobs(&mut canvas, blobs, &self.mode, max_blob_size);
                DynamicImage::ImageRgba8(canvas)
            }
        };

        Ok(FilterResult::Image(output_image))
    }
}

// --- Tiled Bitboard Implementation ---

const TILE_DIM: u32 = 8;
const TILE_SIZE: usize = 64;

// Bitmask constants for 8x8 grid (Row-major)
// Bit 0 is (0,0), Bit 7 is (7,0), Bit 8 is (0,1)
const COL_0_MASK: u64 = 0x0101010101010101;
const COL_7_MASK: u64 = 0x8080808080808080;

struct TileResult {
    /// Maps each bit index (0..63) to a local blob ID. 0 means background.
    labels: [u8; TILE_SIZE],
    /// Number of unique blobs found in this tile
    blob_count: u8,
    /// Coordinates of the tile in the grid (tx, ty)
    tile_pos: (u32, u32),
}

struct BitboardTile {
    pixels_mask: u64,
}

impl BitboardTile {
    fn new(pixels_mask: u64) -> Self {
        Self { pixels_mask }
    }

    fn from_image(
        img: &DynamicImage,
        tx: u32,
        ty: u32,
        threshold: u8,
        target: Option<Rgb<u8>>,
    ) -> Self {
        let mut mask: u64 = 0;
        let start_x = tx * TILE_DIM;
        let start_y = ty * TILE_DIM;
        let width = img.width();
        let height = img.height();

        // Using a flat loop for better instruction pipelining potential
        for i in 0..TILE_SIZE {
            let lx = (i % 8) as u32;
            let ly = (i / 8) as u32;
            let px_x = start_x + lx;
            let px_y = start_y + ly;

            if px_x < width && px_y < height {
                // Inlining the match check logic
                let matches = if let Some(t) = target {
                    let p = img.get_pixel(px_x, px_y).to_rgb();
                    let dr = p.0[0] as i32 - t.0[0] as i32;
                    let dg = p.0[1] as i32 - t.0[1] as i32;
                    let db = p.0[2] as i32 - t.0[2] as i32;
                    (dr * dr + dg * dg + db * db) <= (threshold as i32 * threshold as i32)
                } else {
                    // Use luma
                    let l = img.get_pixel(px_x, px_y).to_luma().0[0];
                    l <= threshold
                };

                if matches {
                    mask |= 1 << i;
                }
            }
        }

        Self::new(mask)
    }

    /// Performs flood fill using bitwise operations entirely on registers/stack.
    fn process(&self, tile_pos: (u32, u32)) -> TileResult {
        let mut remaining = self.pixels_mask;
        let mut labels = [0u8; TILE_SIZE];
        let mut next_id = 1u8;

        while remaining != 0 {
            // Pick the first set bit (Least Significant Bit)
            let seed_idx = remaining.trailing_zeros();
            let seed_mask = 1u64 << seed_idx;

            // Flood fill from this seed
            let mut flood = seed_mask;
            let mut stable = false;

            // Iterative bit propagation
            // This loop usually converges very quickly for 8x8 (8-10 iters typically)
            while !stable {
                let old_flood = flood;

                // Shift & Spread logic
                // North (idx - 8) -> Right Shift 8
                let north = flood >> 8;
                // South (idx + 8) -> Left Shift 8
                let south = flood << 8;
                // East (idx + 1) -> Left Shift 1. Mask out wraparound from Col 7 to next Row.
                // If we shift a bit from Col 7 (0x80) left, it becomes 0x100 (Col 0, Row+1).
                // We must block source bits that are in Col 7 from moving East effectively?
                // Actually: (val << 1) shifts bits East. A bit at Col 7 moves to Col 0 of next row.
                // We want to kill that carry.
                let east = (flood & !COL_7_MASK) << 1;
                // West (idx - 1) -> Right Shift 1.
                let west = (flood & !COL_0_MASK) >> 1;

                flood |= (north | south | east | west) & self.pixels_mask;

                stable = flood == old_flood;
            }

            // Mark found blob
            remaining &= !flood;

            // Assign labels
            // We iterate the set bits in `flood` to fill the labels array.
            // While `flood` is u64, we can use `trailing_zeros` to clear bits efficiently.
            let mut temp_flood = flood;
            while temp_flood != 0 {
                let idx = temp_flood.trailing_zeros();
                labels[idx as usize] = next_id;
                temp_flood &= !(1u64 << idx);
            }

            next_id += 1;
            if next_id == 255 {
                break;
            } // Safety cap, though unlikely for 8x8
        }

        TileResult {
            labels,
            blob_count: next_id - 1,
            tile_pos,
        }
    }
}

struct DisjointSet {
    parent: Vec<usize>,
    size: Vec<usize>,
}

impl DisjointSet {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            size: vec![1; n],
        }
    }

    fn find(&mut self, i: usize) -> usize {
        if self.parent[i] == i {
            return i;
        }
        let root = self.find(self.parent[i]);
        self.parent[i] = root;
        root
    }

    fn union(&mut self, i: usize, j: usize) {
        let root_i = self.find(i);
        let root_j = self.find(j);

        if root_i != root_j {
            if self.size[root_i] < self.size[root_j] {
                self.parent[root_i] = root_j;
                self.size[root_j] += self.size[root_i];
            } else {
                self.parent[root_j] = root_i;
                self.size[root_i] += self.size[root_j];
            }
        }
    }
}

fn find_blobs_tiled(
    img: &DynamicImage,
    threshold: u8,
    target_color: Option<Rgb<u8>>,
) -> BinaryHeap<Blob> {
    let width = img.width();
    let height = img.height();

    if width == 0 || height == 0 {
        return BinaryHeap::new();
    }

    let tiles_x = (width + TILE_DIM - 1) / TILE_DIM;
    let tiles_y = (height + TILE_DIM - 1) / TILE_DIM;

    // 1. Parallel processing of tiles
    // We use Rayon to process tiles in parallel.
    let chunk_results: Vec<TileResult> = (0..tiles_y * tiles_x)
        .into_par_iter()
        .map(|tile_idx| {
            let ty = tile_idx / tiles_x;
            let tx = tile_idx % tiles_x;
            let tile = BitboardTile::from_image(img, tx, ty, threshold, target_color);
            tile.process((tx, ty))
        })
        .collect();

    // We need to map (TileIndex, LocalBlobID) -> GlobalBlobID
    // Total labels could be roughly tiles * avg_blobs_per_tile.
    // We construct offsets.

    // Calculate prefix sums for global ID offsets to map (tile_idx, local_id) -> unique_index
    let mut tile_offsets = vec![0usize; chunk_results.len() + 1];
    for (i, res) in chunk_results.iter().enumerate() {
        tile_offsets[i + 1] = tile_offsets[i] + res.blob_count as usize;
    }
    let total_local_blobs = tile_offsets[chunk_results.len()];

    if total_local_blobs == 0 {
        return BinaryHeap::new();
    }

    let mut dsu = DisjointSet::new(total_local_blobs);

    // Helper to get global ID for a local blob
    let get_global_id = |tile_idx: usize, local_id: u8| -> Option<usize> {
        if local_id == 0 {
            return None;
        }
        // local_id is 1-based, so we subtract 1
        Some(tile_offsets[tile_idx] + (local_id as usize - 1))
    };

    let get_tile_idx = |tx: u32, ty: u32| -> usize { (ty * tiles_x + tx) as usize };

    // Stitching: Iterate over tiles and check Right and Bottom boundaries
    for ty in 0..tiles_y {
        for tx in 0..tiles_x {
            let curr_tile_idx = get_tile_idx(tx, ty);
            let curr_res = &chunk_results[curr_tile_idx];

            // Check Right Boundary (Col 7 of current vs Col 0 of right neighbor)
            if tx + 1 < tiles_x {
                let right_tile_idx = get_tile_idx(tx + 1, ty);
                let right_res = &chunk_results[right_tile_idx];

                for row in 0..8 {
                    // Current tile pixel at (7, row) -> index row*8 + 7
                    let curr_idx = row * 8 + 7;
                    // Right tile pixel at (0, row) -> index row*8 + 0
                    let right_idx = row * 8 + 0;

                    let lid_a = curr_res.labels[curr_idx];
                    let lid_b = right_res.labels[right_idx];

                    if lid_a != 0 && lid_b != 0 {
                        if let (Some(gid_a), Some(gid_b)) = (
                            get_global_id(curr_tile_idx, lid_a),
                            get_global_id(right_tile_idx, lid_b),
                        ) {
                            dsu.union(gid_a, gid_b);
                        }
                    }
                }
            }

            // Check Bottom Boundary (Row 7 of current vs Row 0 of bottom neighbor)
            if ty + 1 < tiles_y {
                let bottom_tile_idx = get_tile_idx(tx, ty + 1);
                let bottom_res = &chunk_results[bottom_tile_idx];

                for col in 0..8 {
                    // Current tile pixel at (col, 7) -> index 7*8 + col = 56 + col
                    let curr_idx = 56 + col;
                    // Bottom tile pixel at (col, 0) -> index 0 + col
                    let bottom_idx = col;

                    let lid_a = curr_res.labels[curr_idx];
                    let lid_b = bottom_res.labels[bottom_idx];

                    if lid_a != 0 && lid_b != 0 {
                        if let (Some(gid_a), Some(gid_b)) = (
                            get_global_id(curr_tile_idx, lid_a),
                            get_global_id(bottom_tile_idx, lid_b),
                        ) {
                            dsu.union(gid_a, gid_b);
                        }
                    }
                }
            }
        }
    }

    // Map root DSU index -> Blob Points
    let mut blob_map: HashMap<usize, Vec<(u32, u32)>> = HashMap::new();

    for (t_idx, res) in chunk_results.iter().enumerate() {
        let start_x = res.tile_pos.0 * TILE_DIM;
        let start_y = res.tile_pos.1 * TILE_DIM;

        // Labels array is O(64) per tile, very fast.
        for i in 0..TILE_SIZE {
            let lid = res.labels[i];
            if lid != 0 {
                let gid = get_global_id(t_idx, lid).unwrap();
                let root = dsu.find(gid);

                let lx = (i % 8) as u32;
                let ly = (i / 8) as u32;
                let gx = start_x + lx;
                let gy = start_y + ly;

                blob_map.entry(root).or_insert_with(Vec::new).push((gx, gy));
            }
        }
    }

    let mut blobs = BinaryHeap::new();
    for points in blob_map.into_values() {
        blobs.push(Blob::new(points));
    }

    blobs
}

trait PixelCanvas {
    fn put_pixel_rgba(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8);
}

impl PixelCanvas for RgbImage {
    fn put_pixel_rgba(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, _a: u8) {
        self.put_pixel(x, y, Rgb([r, g, b]));
    }
}

impl PixelCanvas for RgbaImage {
    fn put_pixel_rgba(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) {
        self.put_pixel(x, y, Rgba([r, g, b, a]));
    }
}

fn draw_blobs<C: PixelCanvas>(
    canvas: &mut C,
    blobs: BinaryHeap<Blob>,
    mode: &BlobColorMode,
    max_blob_size: usize,
) {
    let sorted_blobs = blobs.into_sorted_vec();
    for (i, blob) in sorted_blobs.iter().rev().enumerate() {
        let (r, g, b) = match mode {
            BlobColorMode::Rainbow => {
                let r = ((i * 100 + 50) % 255) as u8;
                let g = ((i * 50 + 100) % 255) as u8;
                let b = ((i * 20 + 150) % 255) as u8;
                (r, g, b)
            }
            BlobColorMode::Heatmap => {
                let size = blob.size;
                if max_blob_size == 0 {
                    (0, 0, 255)
                } else {
                    let t = size as f32 / max_blob_size as f32;
                    // Cool (Blue) -> Hot (Red) -> White
                    if t < 0.8 {
                        // Blue (0,0,255) to Red (255,0,0)
                        let ratio = t / 0.8;
                        let r_val = (255.0 * ratio) as u8;
                        let b_val = (255.0 * (1.0 - ratio)) as u8;
                        (r_val, 0, b_val)
                    } else {
                        // Red (255,0,0) to White (255,255,255)
                        let ratio = (t - 0.8) / 0.2;
                        let gb_val = (255.0 * ratio) as u8;
                        (255, gb_val, gb_val)
                    }
                }
            }
            BlobColorMode::Solid => (0, 255, 0), // Green
        };

        for &(x, y) in &blob.points {
            canvas.put_pixel_rgba(x, y, r, g, b, 255);
        }
    }
}
