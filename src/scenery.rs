//! The tank behind the fish: swaying vines, a toy ship, a skull, a pebble bed, and bubbles.

use std::f64::consts::TAU;

use crate::render::{FRAME_HEIGHT_PIXELS, FRAME_WIDTH_PIXELS, SPRITE_PALETTE, paint_outline};

/// Rows at the bottom of the frame kept for the pebble bed and props; the fish swims above.
pub(crate) const GROUND_RESERVE_PIXELS: f64 = 48.0;

/// The tank is drawn in square cells this many pixels wide, to match a grown fish's pixels.
const CELL_PIXELS: usize = 2;
const GRID_WIDTH: usize = FRAME_WIDTH_PIXELS as usize / CELL_PIXELS;
const GRID_HEIGHT: usize = FRAME_HEIGHT_PIXELS as usize / CELL_PIXELS;

/// A toy galleon, facing right: `.` is clear and digits index [`SPRITE_PALETTE`].
const SHIP_ROWS: [&str; 36] = [
    ".....................1044444................",
    ".....................105555.................",
    ".....................10444..................",
    "..............4444444104444444..............",
    "..............55555551055555544.............",
    "..............55555551055555544.............",
    "..............555555510555555544............",
    "..............55555551055555544.............",
    "..............55555551055555544.10..........",
    "..............4444444104444444..10..........",
    ".....................10.........10..........",
    "............44444444410444444444154.........",
    "............55555555510555555554454.........",
    "............555555555105555555544554........",
    "............555555555105555555544554........",
    "............5555555551055555555545554.......",
    "............5555555551055555555445554.......",
    "............55555555510555555554455554......",
    ".4.4.4.4.4..55555555510555555554455554......",
    ".4444444444.444444444104444444441555554.....",
    ".3333333333..........10.........10......4...",
    ".3333033033..........10.........10......4...",
    ".3333333333..........10.........10.......4..",
    ".3333333333..........10.........10.......4..",
    "..4444444444444444444444444444444444444444..",
    "..2233333333333333333333333333333333333322..",
    "..2233333333344333344333344333344333333322..",
    "...111111111100111100111100111100111111122..",
    "...22333333333333333333333333333333333322...",
    "...22333333333333333333333333333333333322...",
    "....2233333333333333333333333333333333322...",
    ".....11111111111111111111111111111111122....",
    "......223333333333333333333333333333322.....",
    ".......22222222222222222222222222222222.....",
    "........222222222222222222222222222222......",
    ".........1111111111111111111111111122.......",
];
const SKULL_ROWS: [&str; 21] = [
    "........5555555.........",
    "......55555555154.......",
    "....555555555515444.....",
    "...55555555555514443....",
    "..5555555555555144433...",
    "..5555555555555514433...",
    ".555555555555555444333..",
    ".555555555555555444333..",
    ".555500005555500004333..",
    ".555002000555002000333..",
    ".555000000555000000333..",
    "..5500000055500000033...",
    "..5550000555550000433...",
    "...444444440044333333...",
    "...444444440044333333...",
    "...444444440444333333...",
    ".....44444444443333.....",
    ".....40000000000003.....",
    ".....30303030303033.....",
    "......030303030303......",
    "......311111111113......",
];
/// Top-left cells of the ship and the skull, resting in the pebble bed.
const SHIP_CORNER: [i32; 2] = [1, 89];
const SKULL_CORNER: [i32; 2] = [102, 105];

/// The cell row the vines grow from, inside the pebble bed.
const VINE_ROOT_Y: f64 = 123.5;
/// Each vine's root column, height, and phase offset in radians; lengths are in cells.
const VINES: [(f64, f64, f64); 7] = [
    (8.0, 66.0, 0.0),
    (22.0, 44.0, 1.3),
    (52.0, 30.0, 2.9),
    (66.0, 20.0, 4.1),
    (86.0, 38.0, 0.7),
    (98.0, 58.0, 2.2),
    (124.0, 46.0, 3.6),
];
/// Sideways sway at the tip of a vine this tall; shorter vines sway less.
const VINE_SWAY_CELLS_PER_HEIGHT: f64 = 4.0 / 60.0;
const LEAF_SPACING_CELLS: f64 = 6.0;

/// Pebble rows: center line, half-height range, and the first pebble's number, in cells.
const PEBBLE_ROWS: [(f64, [f64; 2], u32); 2] = [(121.5, [1.2, 2.0], 0), (125.5, [1.4, 2.2], 1_000)];

/// Paint the tank, with the vines at the given point in their sway cycle, in radians.
pub(crate) fn paint_scenery(sway_phase: f64) -> Vec<[u8; 4]> {
    let mut cells = vec![[0; 4]; GRID_WIDTH * GRID_HEIGHT];
    for &(root_x, height, phase_offset) in &VINES {
        paint_vine(&mut cells, root_x, height, sway_phase + phase_offset);
    }
    paint_pebble_row(&mut cells, PEBBLE_ROWS[0]);
    paint_sprite(&mut cells, &SHIP_ROWS, SHIP_CORNER);
    paint_sprite(&mut cells, &SKULL_ROWS, SKULL_CORNER);
    paint_pebble_row(&mut cells, PEBBLE_ROWS[1]);
    paint_outline(&mut cells, GRID_WIDTH);
    (0..FRAME_WIDTH_PIXELS as usize * FRAME_HEIGHT_PIXELS as usize)
        .map(|pixel_index| {
            let cell_x = pixel_index % FRAME_WIDTH_PIXELS as usize / CELL_PIXELS;
            let cell_y = pixel_index / FRAME_WIDTH_PIXELS as usize / CELL_PIXELS;
            cells[cell_y * GRID_WIDTH + cell_x]
        })
        .collect()
}

/// Sway phase for a frame of a loop: one whole sway per loop, so the loop closes.
pub(crate) fn calculate_sway_phase(frame_index: u32, loop_frame_count: u32) -> f64 {
    TAU * f64::from(frame_index % loop_frame_count) / f64::from(loop_frame_count)
}

/// A bubble in frame pixels: a light ring with a glint, or one pixel when it is tiny.
pub(crate) fn paint_bubble(rgba_pixels: &mut [[u8; 4]], center: [f64; 2], radius: f64) {
    let frame_size = [FRAME_WIDTH_PIXELS as usize, FRAME_HEIGHT_PIXELS as usize];
    if radius < 1.0 {
        put(
            rgba_pixels,
            frame_size,
            center.map(f64::floor),
            SPRITE_PALETTE[4],
        );
        return;
    }
    let reach = radius.ceil() + 1.0;
    for pixel_y in (center[1] - reach) as i32..=(center[1] + reach) as i32 {
        for pixel_x in (center[0] - reach) as i32..=(center[0] + reach) as i32 {
            let point = [f64::from(pixel_x), f64::from(pixel_y)];
            let distance = (point[0] + 0.5 - center[0]).hypot(point[1] + 0.5 - center[1]);
            if (distance - radius).abs() < 0.6 {
                put(rgba_pixels, frame_size, point, SPRITE_PALETTE[4]);
            }
        }
    }
    let glint = radius * 0.45;
    put(
        rgba_pixels,
        frame_size,
        [(center[0] - glint).floor(), (center[1] - glint).floor()],
        SPRITE_PALETTE[5],
    );
}

fn put(rgba_pixels: &mut [[u8; 4]], size: [usize; 2], point: [f64; 2], color: [u8; 4]) {
    if (0.0..size[0] as f64).contains(&point[0]) && (0.0..size[1] as f64).contains(&point[1]) {
        rgba_pixels[point[1] as usize * size[0] + point[0] as usize] = color;
    }
}

fn put_cell(cells: &mut [[u8; 4]], point: [f64; 2], color: [u8; 4]) {
    put(cells, [GRID_WIDTH, GRID_HEIGHT], point, color);
}

fn paint_sprite(cells: &mut [[u8; 4]], sprite_rows: &[&str], corner: [i32; 2]) {
    for (row_index, sprite_row) in sprite_rows.iter().enumerate() {
        for (column_index, cell) in sprite_row.bytes().enumerate() {
            if let Some(&color) = cell
                .checked_sub(b'0')
                .and_then(|palette_index| SPRITE_PALETTE.get(usize::from(palette_index)))
            {
                put_cell(
                    cells,
                    [
                        f64::from(corner[0]) + column_index as f64,
                        f64::from(corner[1]) + row_index as f64,
                    ],
                    color,
                );
            }
        }
    }
}

/// A stem two cells wide that sways more toward its tip, with leaves on alternate sides.
fn paint_vine(cells: &mut [[u8; 4]], root_x: f64, height: f64, sway_phase: f64) {
    let sway_cells = VINE_SWAY_CELLS_PER_HEIGHT * height;
    let stem_x = |rise: f64| {
        let height_share = rise / height;
        root_x + sway_cells * height_share.powf(1.6) * (sway_phase + 2.2 * height_share).sin()
    };
    let mut rise = 0.0;
    while rise <= height {
        let point = [stem_x(rise).round(), (VINE_ROOT_Y - rise).round()];
        put_cell(cells, point, SPRITE_PALETTE[2]);
        put_cell(cells, [point[0] + 1.0, point[1]], SPRITE_PALETTE[3]);
        rise += 0.5;
    }
    let leaf_count = (height / LEAF_SPACING_CELLS) as usize;
    for leaf_index in 1..=leaf_count {
        let leaf_rise = leaf_index as f64 * LEAF_SPACING_CELLS - 1.0;
        let side = if leaf_index % 2 == 0 { 1.0 } else { -1.0 };
        let leaf_scale = 1.0 - 0.35 * leaf_rise / height;
        paint_leaf(
            cells,
            [
                stem_x(leaf_rise) + 1.0 + side * 3.0 * leaf_scale,
                VINE_ROOT_Y - leaf_rise,
            ],
            side,
            leaf_scale,
        );
    }
}

/// A leaf tilted up and away from the stem, lighter on its upper half.
fn paint_leaf(cells: &mut [[u8; 4]], center: [f64; 2], side: f64, leaf_scale: f64) {
    const LEAF_TILT_RADIANS: f64 = 0.55;
    let half_length = 3.2 * leaf_scale;
    let half_width = 1.4 * leaf_scale.max(0.85);
    let (tilt_sin, tilt_cos) = (side * LEAF_TILT_RADIANS).sin_cos();
    for cell_y in (center[1] - 4.0) as i32..=(center[1] + 4.0) as i32 {
        for cell_x in (center[0] - 5.0) as i32..=(center[0] + 5.0) as i32 {
            let point = [f64::from(cell_x), f64::from(cell_y)];
            let offset_x = point[0] + 0.5 - center[0];
            let offset_y = point[1] + 0.5 - center[1];
            let along = offset_x * tilt_cos + offset_y * tilt_sin;
            let across = -offset_x * tilt_sin + offset_y * tilt_cos;
            if (along / half_length).powi(2) + (across / half_width).powi(2) <= 1.0 {
                let color = if across < 0.0 {
                    SPRITE_PALETTE[5]
                } else {
                    SPRITE_PALETTE[4]
                };
                put_cell(cells, point, color);
            }
        }
    }
}

/// Overlapping rounded pebbles across the tank, shaded from the top left.
fn paint_pebble_row(cells: &mut [[u8; 4]], pebble_row: (f64, [f64; 2], u32)) {
    let (center_y, half_heights, first_pebble) = pebble_row;
    let mut pebble_x = -2.0;
    let mut pebble_index = first_pebble;
    while pebble_x < GRID_WIDTH as f64 + 3.0 {
        let half_height =
            half_heights[0] + (half_heights[1] - half_heights[0]) * hash(pebble_index, 1);
        let half_width = half_height * (1.3 + 0.6 * hash(pebble_index, 2));
        let center = [pebble_x, center_y + hash(pebble_index, 3) - 0.5];
        let base_color = SPRITE_PALETTE[2 + (pebble_index % 2) as usize];
        for cell_y in
            (center[1] - half_height - 1.0) as i32..=(center[1] + half_height + 1.0) as i32
        {
            for cell_x in
                (center[0] - half_width - 1.0) as i32..=(center[0] + half_width + 1.0) as i32
            {
                let point = [f64::from(cell_x), f64::from(cell_y)];
                let across = (point[0] + 0.5 - center[0]) / half_width;
                let down = (point[1] + 0.5 - center[1]) / half_height;
                let distance = across * across + down * down;
                if distance > 1.0 {
                    continue;
                }
                let color = if distance > 0.5 && down > 0.2 {
                    SPRITE_PALETTE[1]
                } else if down < -0.25 && across < 0.15 {
                    SPRITE_PALETTE[4]
                } else {
                    base_color
                };
                put_cell(cells, point, color);
            }
        }
        pebble_x += half_width * 1.7;
        pebble_index += 1;
    }
}

/// A repeatable value from zero up to one for a pebble and one of its traits.
fn hash(pebble_index: u32, trait_index: u32) -> f64 {
    let mut mixed = pebble_index.wrapping_mul(0x9E37_79B9) ^ trait_index.wrapping_mul(0x85EB_CA6B);
    mixed ^= mixed >> 15;
    mixed = mixed.wrapping_mul(0x2C1B_3C6D);
    mixed ^= mixed >> 12;
    f64::from(mixed) / (f64::from(u32::MAX) + 1.0)
}
