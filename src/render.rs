//! A side-view green fish drawn from one sprite, sized by growth, in a tank on a clear canvas.

use std::f64::consts::TAU;

use crate::behavior::Behavior;
use crate::koi::{KoiDevelopment, calculate_growth_fraction};
use crate::motion::{Course, STILL_POSE, SwimPose, calculate_swim_pose, frame_count};
use crate::scenery::{GROUND_RESERVE_PIXELS, calculate_sway_phase, paint_bubble, paint_scenery};

/// Width of the native canvas every fish is drawn on, at every growth level.
pub const FRAME_WIDTH_PIXELS: u32 = 256;
/// Height of the native canvas; the canvas is square.
pub const FRAME_HEIGHT_PIXELS: u32 = FRAME_WIDTH_PIXELS;

/// A renderer-owned, fixed-size frame with row-major RGBA pixels.
#[derive(Debug, Eq, PartialEq)]
pub struct KoiFrame {
    rgba_pixels: Vec<[u8; 4]>,
}

impl KoiFrame {
    /// The pixels as RGBA bytes, row by row from the top left.
    pub fn as_rgba_bytes(&self) -> &[u8] {
        self.rgba_pixels.as_flattened()
    }

    /// Build a frame from chosen pixels, for encoder tests that need colors no fish has.
    #[cfg(test)]
    pub(crate) fn from_rgba_pixels(rgba_pixels: Vec<[u8; 4]>) -> Self {
        Self { rgba_pixels }
    }
}

/// Render the fish in a fixed pose in the still tank, sized by its contribution total.
pub fn render_koi(koi_development: &KoiDevelopment) -> KoiFrame {
    let fish_scale = calculate_fish_scale(koi_development.total_contributions);
    let fish_pixels = render_fish_layer(fish_scale, &STILL_POSE, [1.0; 2]);
    let mut rgba_pixels = compose_over_scenery(&fish_pixels, 0.0);
    let mouth = locate_mouth(
        &place_fish(&STILL_POSE, fish_scale, [1.0; 2]),
        &STILL_POSE,
        fish_scale,
    );
    for (bubble_index, bubble_age) in [10.0, 30.0, 52.0].into_iter().enumerate() {
        paint_rising_bubble(&mut rgba_pixels, mouth, bubble_index, bubble_age);
    }
    KoiFrame { rgba_pixels }
}

/// Render one frame of a behavior's seamless loop; frame numbers wrap around the loop.
pub fn render_swim_frame(
    koi_development: &KoiDevelopment,
    behavior: Behavior,
    frame_index: u32,
) -> KoiFrame {
    const SPAWN_EVERY_FRAMES: u32 = 10;
    const BUBBLE_LIFE_FRAMES: u32 = 150;
    let fish_scale = calculate_fish_scale(koi_development.total_contributions);
    let swim_pose = calculate_swim_pose(behavior, frame_index);
    let drift_scales = fit_hover_drift(behavior, fish_scale);
    let fish_pixels = render_fish_layer(fish_scale, &swim_pose, drift_scales);
    let loop_frame_count = frame_count(behavior);
    let loop_frame = frame_index % loop_frame_count;
    let mut rgba_pixels = compose_over_scenery(
        &fish_pixels,
        calculate_sway_phase(loop_frame, loop_frame_count),
    );
    for spawn_frame in (0..loop_frame_count).step_by(SPAWN_EVERY_FRAMES as usize) {
        let spawn_pose = calculate_swim_pose(behavior, spawn_frame);
        let spawn_mouth = locate_mouth(
            &place_fish(&spawn_pose, fish_scale, drift_scales),
            &spawn_pose,
            fish_scale,
        );
        let first_age = (loop_frame + loop_frame_count - spawn_frame) % loop_frame_count;
        // A bubble outlives a loop, so earlier loops' bubbles may still be rising.
        for bubble_age in (first_age..BUBBLE_LIFE_FRAMES).step_by(loop_frame_count as usize) {
            paint_rising_bubble(
                &mut rgba_pixels,
                spawn_mouth,
                (spawn_frame / SPAWN_EVERY_FRAMES) as usize,
                f64::from(bubble_age),
            );
        }
    }
    KoiFrame { rgba_pixels }
}

/// The fish, facing left: `.` is clear and digits index [`SPRITE_PALETTE`].
const SPRITE_ROWS: [&str; 55] = [
    ".....................................550.....................................",
    "....................................5535.....................................",
    "...................................5522555...................................",
    "..................................553255335..................................",
    ".................................54205223433.................................",
    "................................55005534422254...............................",
    "...............................5500422142125222..............................",
    "...............................52024104..24222454............................",
    "..............................552020022.221024111444.........................",
    "..............................52.1..32..120141044111.........................",
    "..........................................11101111...........................",
    ".........................55555555555555555...2.1...........................55",
    "......................5..2555535552333355555.............................5555",
    "...................555555013435535000013335555..........................5553.",
    ".................5555553510153324340000533033255......................555335.",
    "..............555555555335502225335500035050022155...................5552255.",
    "............555555555543335002200032200055000005155....4555.........5533555..",
    "...........55555555553333350033054033000035000530135.35551.........55225335..",
    ".........5555555555535353200000050000055000000500033502114........551152355..",
    ".......555255555555533510000000005000005000000500000250022.......551153255...",
    "......55535555555552253115554000055000055000055000050355........5511533555...",
    ".....55525553111113552235554500000550003100005000045002255.....5211523555....",
    "....555555531444411555025554550000310000500005000050000502555550005333135....",
    "...5555555514444441355033323335000055000500005000350005000033355041330444....",
    "...555555551445004155503353433500005500052000530051000500050035501001544.....",
    "..555555555144000415550035355350551151505455051505155050005003355.33555......",
    "..555555555144000415555022522330141151505145051505135050505050335.33333......",
    ".3335555555144444413533105322230131151305133051205133050205020313............",
    "55552135555514444133534003221130111151105100051003400050005000243.33444......",
    "1111555225555111135333500224243010045000550005100340005000340242...11444.....",
    "5555112551335555553253400252252010045000350004400042003400042422.1132234.....",
    "1111111135213355553533400223443010033000050000400034000102222230111014444....",
    "35555550025503552353322202245201000300000440004000040001322....11033012444...",
    "3333333550001453333335103244430000330000024000440003330122......1202441344...",
    "11111111355555333335550224440025500000000040000400003112.........330144324...",
    "...3322111113335554420034441013315555550004000033011131...........330144444..",
    ".....233111111332332011350000132333455550004000041122..............33112444..",
    ".......323332110000011111320002412553333000440111233..23............3311244..",
    ".........33233321111111333011003411235555500311122..11032............3343244.",
    "...........3321111333322001112010321115550111233...1013022............442222.",
    "..............333313333301111100113555530013322..3.1421444..............4444.",
    "..................3333101131213300000000011133...331144124................444",
    "....................3..33213133321111133333.......441144444..................",
    "..........................3333333333331............44111444..................",
    ".....................................................444444..................",
    "............................41.4...44....................444.................",
    "............................41.44.444........................................",
    "............................4110411214.......................................",
    "............................11404411444......................................",
    "............................44110441444......................................",
    "............................44144.4414.......................................",
    ".............................4414..44444.....................................",
    "..............................44....4.44.....................................",
    "...............................4.............................................",
    "...............................4.............................................",
];
pub(crate) const SPRITE_PALETTE: [[u8; 4]; 6] = [
    [5, 11, 4, 255],
    [30, 71, 28, 255],
    [47, 111, 38, 255],
    [56, 128, 43, 255],
    [96, 186, 71, 255],
    [132, 217, 101, 255],
];
const SPRITE_WIDTH: usize = SPRITE_ROWS[0].len();
const SPRITE_HEIGHT: usize = SPRITE_ROWS.len();
// Whole cells, so an unscaled, unmoved fish samples every cell at its middle.
const SPRITE_CENTER: [f64; 2] = [(SPRITE_WIDTH / 2) as f64, (SPRITE_HEIGHT / 2) as f64];
/// The closed mouth's line runs along this row from the snout.
const MOUTH_ROW: f64 = 31.5;

const OUTLINE_COLOR: [u8; 4] = SPRITE_PALETTE[0];
const RIPPLE_COLORS: [[u8; 4]; 2] = [[139, 155, 180, 255], [192, 203, 220, 255]];
const DITHER_RANKS: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

/// The body wave starts this many sprite cells behind the snout.
const WAVE_START_CELLS: f64 = 24.0;
const WAVE_LENGTH_CELLS: f64 = 64.0;
/// Tail displacement at wave strength one, in sprite cells.
const WAVE_CELLS: f64 = 1.6;
/// Room above and below the sprite for the wave and tilt, in sprite cells.
const SWAY_CELLS: f64 = 4.0;
/// A fish turning toward or away from the viewer never narrows past this share.
const MIN_TURN_WIDTH: f64 = 0.2;
/// The lap circle is seen from slightly above, so it flattens to this height share.
const LAP_HEIGHT_SHARE: f64 = 0.3;
const MAX_LAP_RADIUS_PIXELS: f64 = 96.0;
/// Room for the outline, a clear pixel at the frame edge, and rounding.
const FIT_MARGIN_PIXELS: f64 = 2.5;

/// Native pixels per sprite cell, from the smallest fish to the largest.
fn calculate_fish_scale(total_contributions: u64) -> f64 {
    const MIN_FISH_SCALE: f64 = 1.0;
    const MAX_FISH_SCALE: f64 = 2.0;
    interpolate(
        MIN_FISH_SCALE,
        MAX_FISH_SCALE,
        calculate_growth_fraction(total_contributions),
    )
}

/// The middle of the water above the pebble bed, where the fish swims around.
const SWIM_CENTER: [f64; 2] = [
    FRAME_WIDTH_PIXELS as f64 / 2.0,
    (FRAME_HEIGHT_PIXELS as f64 - GROUND_RESERVE_PIXELS) / 2.0,
];
/// How far the fish may reach from the swim center on each axis.
const SWIM_REACH: [f64; 2] = [
    SWIM_CENTER[0] - FIT_MARGIN_PIXELS,
    SWIM_CENTER[1] - FIT_MARGIN_PIXELS,
];

/// Where the fish is in one frame and how it faces.
struct Placement {
    /// Sprite center, from the swim center, in native pixels.
    center: [f64; 2],
    /// Horizontal stretch: one faces left as drawn, negative mirrors it to face right.
    facing_scale: f64,
    /// Rise of the head over the tail per sprite cell; positive lifts the head.
    tilt: f64,
}

/// Half the fish's width and height in native pixels, with room for its sway.
fn measure_half_extents(fish_scale: f64) -> [f64; 2] {
    [
        (SPRITE_WIDTH as f64 - SPRITE_CENTER[0]) * fish_scale,
        (SPRITE_HEIGHT as f64 - SPRITE_CENTER[1] + SWAY_CELLS) * fish_scale,
    ]
}

/// Per axis, the share of the loop's drift that keeps every hovering frame inside.
fn fit_hover_drift(behavior: Behavior, fish_scale: f64) -> [f64; 2] {
    let half_extents = measure_half_extents(fish_scale);
    let mut largest_drift = [0.0_f64; 2];
    for frame_index in 0..frame_count(behavior) {
        if let Course::Hover { drift_pixels, .. } =
            calculate_swim_pose(behavior, frame_index).course
        {
            for axis_index in 0..2 {
                largest_drift[axis_index] =
                    largest_drift[axis_index].max(drift_pixels[axis_index].abs());
            }
        }
    }
    [0, 1].map(|axis_index| {
        let room = SWIM_REACH[axis_index] - half_extents[axis_index];
        if largest_drift[axis_index] == 0.0 {
            1.0
        } else {
            (room / largest_drift[axis_index]).clamp(0.0, 1.0)
        }
    })
}

/// Lap ellipse radii that fit a full-width fish mid-lap and a narrowed one at the sides.
fn calculate_lap_radii(fish_scale: f64) -> [f64; 2] {
    let [half_width, half_height] = measure_half_extents(fish_scale);
    let [reach_x, reach_y] = SWIM_REACH;
    let radius_x = (reach_x * reach_x - half_width * half_width)
        .max(0.0)
        .sqrt()
        .min(reach_x - half_width * MIN_TURN_WIDTH)
        .min(MAX_LAP_RADIUS_PIXELS);
    let radius_y = (radius_x * LAP_HEIGHT_SHARE).min((reach_y - half_height).max(0.0));
    [radius_x, radius_y]
}

fn place_fish(swim_pose: &SwimPose, fish_scale: f64, drift_scales: [f64; 2]) -> Placement {
    match swim_pose.course {
        Course::Hover {
            heading_offset_radians,
            drift_pixels,
        } => Placement {
            center: [0, 1].map(|axis_index| drift_pixels[axis_index] * drift_scales[axis_index]),
            facing_scale: 1.0,
            tilt: heading_offset_radians.tan(),
        },
        Course::Lap { angle_radians } => {
            let [radius_x, radius_y] = calculate_lap_radii(fish_scale);
            // The clockwise heading's horizontal share; positive is rightward.
            let heading_x = -angle_radians.sin();
            Placement {
                center: [
                    radius_x * angle_radians.cos(),
                    radius_y * angle_radians.sin(),
                ],
                facing_scale: -heading_x.signum() * heading_x.abs().max(MIN_TURN_WIDTH),
                tilt: 0.0,
            }
        }
    }
}

/// Vertical shift of a sprite column, in cells: the body wave plus the tilt.
fn calculate_column_bend(column: f64, swim_pose: &SwimPose, tilt: f64) -> f64 {
    let wave_weight =
        smooth_step((column - WAVE_START_CELLS) / (SPRITE_WIDTH as f64 - WAVE_START_CELLS));
    let wave = WAVE_CELLS
        * swim_pose.wave_strength
        * wave_weight
        * (swim_pose.wave_phase_radians - TAU * column / WAVE_LENGTH_CELLS).sin();
    wave + tilt * (column - SPRITE_CENTER[0])
}

/// Whether an open mouth clears this sprite point: a wedge from the snout on the mouth line.
fn is_in_open_mouth(sprite_point: [f64; 2], mouth_opening: f64) -> bool {
    const VISIBLE_OPENING: f64 = 0.1;
    const DEPTH_CELLS: f64 = 7.0;
    const GAPE_CELLS: f64 = 2.5;
    let [column, row] = sprite_point;
    mouth_opening > VISIBLE_OPENING
        && (0.0..DEPTH_CELLS).contains(&column)
        && (row - MOUTH_ROW).abs()
            <= 0.5 + mouth_opening.min(1.0) * GAPE_CELLS * (1.0 - column / DEPTH_CELLS)
}

fn sample_sprite(column: f64, row: f64) -> [u8; 4] {
    if column < 0.0 || row < 0.0 {
        return [0; 4];
    }
    SPRITE_ROWS
        .get(row as usize)
        .and_then(|sprite_row| sprite_row.as_bytes().get(column as usize))
        .and_then(|&cell| cell.checked_sub(b'0'))
        .and_then(|palette_index| SPRITE_PALETTE.get(usize::from(palette_index)))
        .copied()
        .unwrap_or([0; 4])
}

/// Where the mouth is in the frame, in native pixels.
fn locate_mouth(placement: &Placement, swim_pose: &SwimPose, fish_scale: f64) -> [f64; 2] {
    let mouth_bend = calculate_column_bend(0.0, swim_pose, placement.tilt);
    [
        SWIM_CENTER[0] + placement.center[0]
            - SPRITE_CENTER[0] * fish_scale * placement.facing_scale,
        SWIM_CENTER[1]
            + placement.center[1]
            + (MOUTH_ROW - SPRITE_CENTER[1] + mouth_bend) * fish_scale,
    ]
}

/// The tank with the fish layer drawn over it.
fn compose_over_scenery(fish_pixels: &[[u8; 4]], sway_phase: f64) -> Vec<[u8; 4]> {
    let mut rgba_pixels = paint_scenery(sway_phase);
    for (scene_pixel, &fish_pixel) in rgba_pixels.iter_mut().zip(fish_pixels) {
        if fish_pixel[3] != 0 {
            *scene_pixel = fish_pixel;
        }
    }
    rgba_pixels
}

/// A bubble rising from the mouth with a wobble, growing a little, until it reaches the top.
fn paint_rising_bubble(
    rgba_pixels: &mut [[u8; 4]],
    mouth: [f64; 2],
    bubble_index: usize,
    bubble_age: f64,
) {
    const RISE_PIXELS_PER_FRAME: f64 = 1.3;
    const WOBBLE_PIXELS: f64 = 1.5;
    const RADII: [f64; 3] = [2.5, 1.5, 3.5];
    let radius = RADII[bubble_index % RADII.len()] * (0.5 + bubble_age / 24.0).min(1.0);
    let center = [
        mouth[0] + WOBBLE_PIXELS * (0.3 * bubble_age + bubble_index as f64 * 1.9).sin(),
        mouth[1] - 3.0 - RISE_PIXELS_PER_FRAME * bubble_age,
    ];
    if center[1] - radius > 2.0 {
        paint_bubble(rgba_pixels, center, radius);
    }
}

/// The fish alone, outlined, with any feeding extras and depth tint, on a clear canvas.
fn render_fish_layer(
    fish_scale: f64,
    swim_pose: &SwimPose,
    drift_scales: [f64; 2],
) -> Vec<[u8; 4]> {
    let placement = place_fish(swim_pose, fish_scale, drift_scales);
    let mut rgba_pixels = Vec::with_capacity((FRAME_WIDTH_PIXELS * FRAME_HEIGHT_PIXELS) as usize);
    for pixel_y in 0..FRAME_HEIGHT_PIXELS {
        for pixel_x in 0..FRAME_WIDTH_PIXELS {
            let offset = [
                f64::from(pixel_x) + 0.5 - SWIM_CENTER[0] - placement.center[0],
                f64::from(pixel_y) + 0.5 - SWIM_CENTER[1] - placement.center[1],
            ];
            let column = offset[0] / (fish_scale * placement.facing_scale) + SPRITE_CENTER[0];
            let row = offset[1] / fish_scale + SPRITE_CENTER[1]
                - calculate_column_bend(column, swim_pose, placement.tilt);
            rgba_pixels.push(
                if is_in_open_mouth([column, row], swim_pose.mouth_opening) {
                    [0; 4]
                } else {
                    sample_sprite(column, row)
                },
            );
        }
    }
    let fish_pixel_mask: Vec<bool> = rgba_pixels
        .iter()
        .map(|pixel_channels| pixel_channels[3] != 0)
        .collect();
    paint_outline(&mut rgba_pixels, FRAME_WIDTH_PIXELS as usize);
    if let Some(feeding_phase) = swim_pose.feeding_phase {
        paint_feeding(
            &mut rgba_pixels,
            locate_mouth(&placement, swim_pose, fish_scale),
            [-placement.facing_scale.signum(), 0.0],
            SPRITE_WIDTH as f64 * fish_scale,
            feeding_phase,
        );
    }
    if swim_pose.depth > 0.0 {
        paint_depth(&mut rgba_pixels, &fish_pixel_mask, swim_pose.depth);
    }
    rgba_pixels
}

/// Ripples spreading from the mouth and pellets drifting into it, shrunk to fit the frame.
fn paint_feeding(
    rgba_pixels: &mut [[u8; 4]],
    mouth: [f64; 2],
    forward_direction: [f64; 2],
    body_length_pixels: f64,
    feeding_phase: f64,
) {
    const RING_COUNT: usize = 3;
    const RING_WIDTH_PER_BODY_LENGTH: f64 = 0.3;
    const RING_HEIGHT_PER_WIDTH: f64 = 0.4;
    const RING_ANGLE_STEP_RADIANS: f64 = 0.035;
    const PELLET_COUNT: usize = 5;
    const PELLET_NEAREST_PIXELS: f64 = 4.0;
    const PELLET_REACH_PIXELS: f64 = 14.0;
    const PELLET_WANDER_PIXELS: f64 = 9.0;
    // Pellets vanish just before they reach the mouth.
    const PELLET_EATEN_LIFE: f64 = 0.92;
    const PELLET_COLORS: [[u8; 4]; 2] = [[228, 166, 114, 255], [184, 111, 80, 255]];
    // A clear pixel between the extras and the frame edge, plus rounding.
    const EDGE_MARGIN_PIXELS: f64 = 2.0;
    let frame_sizes = [
        f64::from(FRAME_WIDTH_PIXELS),
        f64::from(FRAME_HEIGHT_PIXELS),
    ];
    let room = [0, 1].map(|axis_index| {
        (mouth[axis_index] - EDGE_MARGIN_PIXELS)
            .min(frame_sizes[axis_index] - 1.0 - EDGE_MARGIN_PIXELS - mouth[axis_index])
            .max(0.0)
    });
    let frame_width = FRAME_WIDTH_PIXELS as usize;
    let mut paint_water = |point: [f64; 2], color: [u8; 4], dither_limit: f64| {
        let [pixel_x, pixel_y] = point.map(f64::floor);
        if !(0.0..frame_sizes[0]).contains(&pixel_x) || !(0.0..frame_sizes[1]).contains(&pixel_y) {
            return;
        }
        let pixel_index = pixel_y as usize * frame_width + pixel_x as usize;
        if rgba_pixels[pixel_index][3] == 0
            && calculate_dither_threshold([pixel_x as u32, pixel_y as u32]) < dither_limit
        {
            rgba_pixels[pixel_index] = color;
        }
    };
    let largest_ring_width = body_length_pixels * RING_WIDTH_PER_BODY_LENGTH * 0.85;
    let ring_scale = 1.0_f64
        .min(room[0] / largest_ring_width)
        .min(room[1] / (largest_ring_width * RING_HEIGHT_PER_WIDTH));
    for ring_index in 0..RING_COUNT {
        let ring_phase = (feeding_phase + ring_index as f64 / RING_COUNT as f64).fract();
        let ring_width = ring_scale
            * body_length_pixels
            * RING_WIDTH_PER_BODY_LENGTH
            * (0.25 + 0.6 * ring_phase);
        let ring_color = RIPPLE_COLORS[ring_index % 2];
        let mut ring_angle: f64 = 0.0;
        while ring_angle < TAU {
            paint_water(
                [
                    mouth[0] + ring_angle.cos() * ring_width,
                    mouth[1] + ring_angle.sin() * ring_width * RING_HEIGHT_PER_WIDTH,
                ],
                ring_color,
                (1.0 - ring_phase) * 0.75,
            );
            ring_angle += RING_ANGLE_STEP_RADIANS;
        }
    }
    // The second pellet pixel sits one to the right, so leave it room too.
    let farthest_pellet_pixels = PELLET_NEAREST_PIXELS + PELLET_REACH_PIXELS + PELLET_WANDER_PIXELS;
    let pellet_scale = 1.0_f64.min((room[0].min(room[1]) - 1.0).max(0.0) / farthest_pellet_pixels);
    for pellet_index in 0..PELLET_COUNT {
        let life = (feeding_phase + pellet_index as f64 / PELLET_COUNT as f64).fract();
        if life >= PELLET_EATEN_LIFE {
            continue;
        }
        let reach = PELLET_NEAREST_PIXELS + (1.0 - life) * PELLET_REACH_PIXELS;
        let wander = (1.0 - life) * PELLET_WANDER_PIXELS;
        let pellet_point = [
            mouth[0]
                + pellet_scale
                    * (forward_direction[0] * reach + (pellet_index as f64 * 2.1).sin() * wander),
            mouth[1]
                + pellet_scale
                    * (forward_direction[1] * reach + (pellet_index as f64 * 1.7).cos() * wander),
        ];
        paint_water(pellet_point, PELLET_COLORS[0], 1.0);
        paint_water(
            [pellet_point[0] + 1.0, pellet_point[1]],
            PELLET_COLORS[1],
            1.0,
        );
    }
}

/// Under water, tint toward water blue by brightness and drop out a dithered share.
fn paint_depth(rgba_pixels: &mut [[u8; 4]], fish_pixel_mask: &[bool], depth: f64) {
    const DEEP_WATER: [f64; 3] = [18.0, 78.0, 137.0];
    const MID_WATER: [f64; 3] = [0.0, 153.0, 219.0];
    const SHALLOW_WATER: [f64; 3] = [190.0, 225.0, 245.0];
    const OUTLINE_WATER: [f64; 3] = [24.0, 48.0, 92.0];
    const FISH_DROPOUT_PER_DEPTH: f64 = 0.15;
    const OUTLINE_DROPOUT_PER_DEPTH: f64 = 0.45;
    const TINT_PER_DEPTH: f64 = 1.15;
    let tint = (depth * TINT_PER_DEPTH).min(1.0);
    let frame_width = FRAME_WIDTH_PIXELS as usize;
    for (pixel_index, pixel_channels) in rgba_pixels.iter_mut().enumerate() {
        if pixel_channels[3] == 0 {
            continue;
        }
        let is_fish_pixel = fish_pixel_mask[pixel_index];
        let dropout_per_depth = if is_fish_pixel {
            FISH_DROPOUT_PER_DEPTH
        } else {
            OUTLINE_DROPOUT_PER_DEPTH
        };
        let pixel_position = [
            (pixel_index % frame_width) as u32,
            (pixel_index / frame_width) as u32,
        ];
        if calculate_dither_threshold(pixel_position) < depth * dropout_per_depth {
            *pixel_channels = [0; 4];
            continue;
        }
        let original_color =
            [0, 1, 2].map(|channel_index| f64::from(pixel_channels[channel_index]));
        let water_color = if is_fish_pixel {
            let brightness =
                (0.3 * original_color[0] + 0.59 * original_color[1] + 0.11 * original_color[2])
                    / 255.0;
            let (from_color, to_color, blend) = if brightness < 0.5 {
                (DEEP_WATER, MID_WATER, brightness * 2.0)
            } else {
                (MID_WATER, SHALLOW_WATER, brightness * 2.0 - 1.0)
            };
            [0, 1, 2].map(|channel_index| {
                interpolate(from_color[channel_index], to_color[channel_index], blend)
            })
        } else {
            OUTLINE_WATER
        };
        for channel_index in 0..3 {
            pixel_channels[channel_index] = interpolate(
                original_color[channel_index],
                water_color[channel_index],
                tint,
            )
            .round() as u8;
        }
    }
}

/// Outline every shape in a row-major image of the given width with one dark pixel.
pub(crate) fn paint_outline(rgba_pixels: &mut [[u8; 4]], frame_width: usize) {
    let original_pixels = rgba_pixels.to_vec();
    let frame_height = rgba_pixels.len() / frame_width;
    for (pixel_index, pixel_channels) in rgba_pixels.iter_mut().enumerate() {
        if original_pixels[pixel_index][3] != 0 {
            continue;
        }
        let pixel_x = pixel_index % frame_width;
        let pixel_y = pixel_index / frame_width;
        let touches_shape = (pixel_x > 0 && original_pixels[pixel_index - 1][3] != 0)
            || (pixel_x + 1 < frame_width && original_pixels[pixel_index + 1][3] != 0)
            || (pixel_y > 0 && original_pixels[pixel_index - frame_width][3] != 0)
            || (pixel_y + 1 < frame_height && original_pixels[pixel_index + frame_width][3] != 0);
        if touches_shape {
            *pixel_channels = OUTLINE_COLOR;
        }
    }
}

fn interpolate(from: f64, to: f64, fraction: f64) -> f64 {
    from + (to - from) * fraction
}

/// Ease in and out across zero through one; values outside are clamped.
fn smooth_step(fraction: f64) -> f64 {
    let fraction = fraction.clamp(0.0, 1.0);
    fraction * fraction * (3.0 - 2.0 * fraction)
}

/// The ordered-dither threshold at a pixel, from zero through one.
fn calculate_dither_threshold(pixel_position: [u32; 2]) -> f64 {
    let dither_x = pixel_position[0] as usize % DITHER_RANKS[0].len();
    let dither_y = pixel_position[1] as usize % DITHER_RANKS.len();
    (f64::from(DITHER_RANKS[dither_y][dither_x]) + 0.5)
        / (DITHER_RANKS.len() * DITHER_RANKS[0].len()) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sprite_row_is_as_wide_as_the_first() {
        assert!(
            SPRITE_ROWS
                .iter()
                .all(|sprite_row| sprite_row.len() == SPRITE_WIDTH)
        );
    }

    #[test]
    fn the_mouth_opens_as_a_wedge_on_the_mouth_line() {
        let snout_point = [1.5, MOUTH_ROW];
        assert!(!is_in_open_mouth(snout_point, 0.1));
        assert!(is_in_open_mouth(snout_point, 0.5));
        let cleared_cells = |mouth_opening| {
            (0..SPRITE_HEIGHT)
                .flat_map(|row| (0..SPRITE_WIDTH).map(move |column| (column, row)))
                .filter(|&(column, row)| {
                    is_in_open_mouth([column as f64 + 0.5, row as f64 + 0.5], mouth_opening)
                })
                .count()
        };
        assert!(0 < cleared_cells(0.5) && cleared_cells(0.5) < cleared_cells(1.0));
    }

    const BEHAVIORS: [Behavior; 5] = [
        Behavior::Cruise,
        Behavior::Dart,
        Behavior::Rest,
        Behavior::SurfaceFeeding,
        Behavior::DeepRest,
    ];

    fn render_swimming_fish(
        total_contributions: u64,
        behavior: Behavior,
        frame: u32,
    ) -> Vec<[u8; 4]> {
        let fish_scale = calculate_fish_scale(total_contributions);
        render_fish_layer(
            fish_scale,
            &calculate_swim_pose(behavior, frame),
            fit_hover_drift(behavior, fish_scale),
        )
    }

    /// The `[left, top, right, bottom]` pixels around everything drawn.
    fn measure_bounds(rgba_pixels: &[[u8; 4]]) -> [usize; 4] {
        let frame_width = FRAME_WIDTH_PIXELS as usize;
        let mut bounds = [usize::MAX, usize::MAX, 0, 0];
        for (pixel_index, _) in rgba_pixels
            .iter()
            .enumerate()
            .filter(|(_, pixel)| pixel[3] != 0)
        {
            let [pixel_x, pixel_y] = [pixel_index % frame_width, pixel_index / frame_width];
            bounds = [
                bounds[0].min(pixel_x),
                bounds[1].min(pixel_y),
                bounds[2].max(pixel_x),
                bounds[3].max(pixel_y),
            ];
        }
        assert!(bounds[0] < bounds[2] && bounds[1] < bounds[3]);
        bounds
    }

    fn measure_centroid(rgba_pixels: &[[u8; 4]]) -> [f64; 2] {
        let frame_width = FRAME_WIDTH_PIXELS as usize;
        let drawn: Vec<usize> = (0..rgba_pixels.len())
            .filter(|&pixel_index| rgba_pixels[pixel_index][3] != 0)
            .collect();
        let count = drawn.len() as f64;
        [
            drawn
                .iter()
                .map(|&pixel_index| (pixel_index % frame_width) as f64)
                .sum::<f64>()
                / count,
            drawn
                .iter()
                .map(|&pixel_index| (pixel_index / frame_width) as f64)
                .sum::<f64>()
                / count,
        ]
    }

    fn is_inside_water(bounds: [usize; 4]) -> bool {
        let water_bottom = (f64::from(FRAME_HEIGHT_PIXELS) - GROUND_RESERVE_PIXELS) as usize;
        bounds[0] > 0
            && bounds[1] > 0
            && bounds[2] < FRAME_WIDTH_PIXELS as usize - 1
            && bounds[3] < water_bottom
    }

    #[test]
    fn growth_slows_and_keeps_the_fish_in_the_water_above_the_ground() {
        let widths: Vec<(u64, usize)> = [0, 500, 1_500, 2_000, 12_750, 50_000, 100_000, u64::MAX]
            .into_iter()
            .map(|total_contributions| {
                let fish_pixels = render_fish_layer(
                    calculate_fish_scale(total_contributions),
                    &STILL_POSE,
                    [1.0; 2],
                );
                let bounds = measure_bounds(&fish_pixels);
                assert!(is_inside_water(bounds), "{total_contributions}: {bounds:?}");
                (total_contributions, bounds[2] - bounds[0])
            })
            .collect();
        assert!(widths.windows(2).all(|pair| pair[0].1 <= pair[1].1));
        assert!(widths[1].1 - widths[0].1 > widths[3].1 - widths[2].1);
        assert_eq!(widths[6].1, widths[7].1);
    }

    #[test]
    fn swimming_keeps_the_fish_in_the_water_above_the_ground() {
        for behavior in BEHAVIORS {
            for total_contributions in [0, 1_500, 12_750, 100_000] {
                for frame in (0..frame_count(behavior)).step_by(5) {
                    let bounds =
                        measure_bounds(&render_swimming_fish(total_contributions, behavior, frame));
                    assert!(
                        is_inside_water(bounds),
                        "{behavior:?} at {total_contributions}, frame {frame}: {bounds:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn lapping_fish_face_the_way_they_swim() {
        // The deep head outweighs the tail, so the centroid leans toward the head.
        let head_side = |fish_pixels: &[[u8; 4]]| {
            let bounds = measure_bounds(fish_pixels);
            measure_centroid(fish_pixels)[0] - (bounds[0] + bounds[2]) as f64 / 2.0
        };
        let half_loop = frame_count(Behavior::Cruise) / 2;
        assert!(head_side(&render_fish_layer(1.5, &STILL_POSE, [1.0; 2])) < 0.0);
        assert!(head_side(&render_swimming_fish(12_750, Behavior::Cruise, 0)) > 0.0);
        assert!(head_side(&render_swimming_fish(12_750, Behavior::Cruise, half_loop)) < 0.0);
    }

    #[test]
    fn deeper_rest_turns_the_fish_toward_water_blue() {
        let blue_lean = |behavior| {
            let fish_pixels = render_swimming_fish(12_750, behavior, 0);
            let drawn: Vec<_> = fish_pixels.iter().filter(|pixel| pixel[3] != 0).collect();
            drawn
                .iter()
                .map(|pixel| f64::from(pixel[2]) - f64::from(pixel[1]))
                .sum::<f64>()
                / drawn.len() as f64
        };
        let [cruise, rest, deep_rest] =
            [Behavior::Cruise, Behavior::Rest, Behavior::DeepRest].map(blue_lean);
        assert!(
            rest > cruise + 10.0 && deep_rest > rest + 10.0,
            "{cruise} {rest} {deep_rest}"
        );
    }

    #[test]
    fn the_fish_never_jumps_or_vanishes() {
        for behavior in BEHAVIORS {
            let frames: Vec<_> = (0..=frame_count(behavior))
                .map(|frame| render_swimming_fish(1_500, behavior, frame))
                .collect();
            // Darting bursts are fast by design; every other behavior moves gently.
            if behavior != Behavior::Dart {
                for pair in frames.windows(2) {
                    let [before, after] =
                        [&pair[0], &pair[1]].map(|pixels| measure_centroid(pixels));
                    let step = (after[0] - before[0]).hypot(after[1] - before[1]);
                    assert!(step < 8.0, "{behavior:?} jumps {step}");
                }
            }
            let counts: Vec<usize> = frames
                .iter()
                .map(|pixels| pixels.iter().filter(|pixel| pixel[3] != 0).count())
                .collect();
            let largest = *counts.iter().max().unwrap();
            assert!(
                counts.iter().all(|&count| count * 8 > largest),
                "{behavior:?}"
            );
        }
    }
}
