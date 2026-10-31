use std::collections::{BTreeMap, BTreeSet};
use std::io::Cursor;

use chrono::Days;
use git_koi::behavior::Behavior;
use git_koi::history::{AccountSnapshot, ContributionDay};
use git_koi::koi::{
    ADULT_CONTRIBUTIONS, KoiDevelopment, MAX_GROWTH_CONTRIBUTIONS, derive_koi_development,
};
use git_koi::motion::frame_count;
use git_koi::output::encode_png;
use git_koi::render::{
    FRAME_HEIGHT_PIXELS, FRAME_WIDTH_PIXELS, KoiFrame, render_koi, render_swim_frame,
};

const BEHAVIORS: [Behavior; 5] = [
    Behavior::Cruise,
    Behavior::Dart,
    Behavior::Rest,
    Behavior::SurfaceFeeding,
    Behavior::DeepRest,
];

#[test]
fn languages_and_accounts_do_not_change_the_fish() {
    let koi_development = develop_with_total(1_500);
    let mut other_development = develop_with_total(1_500);
    other_development.language_shares.clear();
    other_development.language_colors.clear();
    other_development.identity_seed += 1;
    assert_eq!(render_koi(&koi_development), render_koi(&other_development));
}

#[test]
fn growth_slows_and_keeps_the_whole_fish_inside_a_transparent_frame() {
    let mut koi_development = read_fixture_development();
    let mut visible_widths = BTreeMap::new();
    let mut maximum_frame = None;
    for contribution_total in [
        0,
        500,
        1_000,
        1_500,
        2_000,
        12_750,
        25_000,
        50_000,
        75_000,
        100_000,
        u64::MAX,
    ] {
        koi_development.total_contributions = contribution_total;
        let koi_frame = render_koi(&koi_development);
        let opaque_bounds = measure_opaque_bounds(&koi_frame);
        assert!(opaque_bounds.left_x > 0 && opaque_bounds.top_y > 0);
        assert!(opaque_bounds.right_x < FRAME_WIDTH_PIXELS - 1);
        assert!(opaque_bounds.bottom_y < FRAME_HEIGHT_PIXELS - 1);
        visible_widths.insert(
            contribution_total,
            opaque_bounds.right_x - opaque_bounds.left_x + 1,
        );
        if contribution_total == 100_000 {
            maximum_frame = Some(koi_frame);
        } else if contribution_total > 100_000 {
            assert_eq!(Some(koi_frame), maximum_frame);
        }
    }
    let growth_widths: Vec<_> = visible_widths.values().copied().collect();
    assert!(
        growth_widths
            .windows(2)
            .all(|adjacent_widths| adjacent_widths[0] <= adjacent_widths[1])
    );
    assert!(
        visible_widths[&500] - visible_widths[&0] > visible_widths[&2_000] - visible_widths[&1_500]
    );
    for contribution_pair in [12_750, 25_000, 50_000, 75_000, 100_000].windows(2) {
        assert!(visible_widths[&contribution_pair[0]] < visible_widths[&contribution_pair[1]]);
    }
}

#[test]
fn swimming_keeps_the_whole_fish_inside_the_frame() {
    let mut koi_development = read_fixture_development();
    for behavior in BEHAVIORS {
        for contribution_total in [0, 1_500, ADULT_CONTRIBUTIONS, MAX_GROWTH_CONTRIBUTIONS] {
            koi_development.total_contributions = contribution_total;
            for frame_index in (0..frame_count(behavior)).step_by(5) {
                let opaque_bounds = measure_opaque_bounds(&render_swim_frame(
                    &koi_development,
                    behavior,
                    frame_index,
                ));
                assert!(
                    opaque_bounds.left_x > 0
                        && opaque_bounds.top_y > 0
                        && opaque_bounds.right_x < FRAME_WIDTH_PIXELS - 1
                        && opaque_bounds.bottom_y < FRAME_HEIGHT_PIXELS - 1,
                    "{behavior:?} at {contribution_total} contributions leaves the frame at \
                     frame {frame_index}"
                );
            }
        }
    }
}

#[test]
fn lapping_fish_face_the_way_they_swim() {
    let koi_development = develop_with_total(ADULT_CONTRIBUTIONS);
    let half_loop = frame_count(Behavior::Cruise) / 2;
    let head_side = |frame_index| {
        let swim_frame = render_swim_frame(&koi_development, Behavior::Cruise, frame_index);
        let opaque_bounds = measure_opaque_bounds(&swim_frame);
        let middle_x = f64::from(opaque_bounds.left_x + opaque_bounds.right_x) / 2.0;
        measure_opaque_centroid(&swim_frame)[0] - middle_x
    };
    let still_side = {
        let still_frame = render_koi(&koi_development);
        let opaque_bounds = measure_opaque_bounds(&still_frame);
        measure_opaque_centroid(&still_frame)[0]
            - f64::from(opaque_bounds.left_x + opaque_bounds.right_x) / 2.0
    };
    // The deep head outweighs the tail, so the centroid leans toward the head.
    assert!(still_side < 0.0);
    assert!(head_side(0) > 0.0, "the top of the lap heads right");
    assert!(
        head_side(half_loop) < 0.0,
        "the bottom of the lap heads left"
    );
}

#[test]
fn surface_feeding_draws_pellets_that_rest_does_not() {
    const PELLET_COLOR: [u8; 4] = [228, 166, 114, 255];
    let koi_development = develop_with_total(1_500);
    let colors_of = |behavior| {
        (0..frame_count(behavior))
            .step_by(5)
            .flat_map(|frame_index| {
                collect_opaque_colors(&render_swim_frame(&koi_development, behavior, frame_index))
            })
            .collect::<BTreeSet<_>>()
    };
    assert!(colors_of(Behavior::SurfaceFeeding).contains(&PELLET_COLOR));
    assert!(!colors_of(Behavior::Rest).contains(&PELLET_COLOR));
}

#[test]
fn deeper_rest_turns_the_fish_toward_water_blue() {
    let koi_development = develop_with_total(ADULT_CONTRIBUTIONS);
    let measure_blue_lean = |behavior| {
        let swim_frame = render_swim_frame(&koi_development, behavior, 0);
        let opaque_pixels: Vec<_> = swim_frame
            .as_rgba_bytes()
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel_channels| pixel_channels[3] != 0)
            .copied()
            .collect();
        opaque_pixels
            .iter()
            .map(|pixel_channels| f64::from(pixel_channels[2]) - f64::from(pixel_channels[1]))
            .sum::<f64>()
            / opaque_pixels.len() as f64
    };
    let cruise_lean = measure_blue_lean(Behavior::Cruise);
    let rest_lean = measure_blue_lean(Behavior::Rest);
    let deep_rest_lean = measure_blue_lean(Behavior::DeepRest);
    assert!(
        rest_lean > cruise_lean + 10.0,
        "{cruise_lean} then {rest_lean}"
    );
    assert!(
        deep_rest_lean > rest_lean + 10.0,
        "{rest_lean} then {deep_rest_lean}"
    );
}

#[test]
fn the_swim_loop_is_seamless_and_never_jumps() {
    let koi_development = develop_with_total(1_500);
    for behavior in BEHAVIORS {
        let loop_frame_count = frame_count(behavior);
        let swim_frames: Vec<_> = (0..=loop_frame_count)
            .map(|frame_index| render_swim_frame(&koi_development, behavior, frame_index))
            .collect();
        assert_eq!(swim_frames[loop_frame_count as usize], swim_frames[0]);
        let centroids: Vec<_> = swim_frames.iter().map(measure_opaque_centroid).collect();
        // Darting bursts are fast by design; every other behavior moves gently.
        for centroid_pair in centroids.windows(2).filter(|_| behavior != Behavior::Dart) {
            let centroid_step = (centroid_pair[1][0] - centroid_pair[0][0])
                .hypot(centroid_pair[1][1] - centroid_pair[0][1]);
            assert!(centroid_step < 8.0, "{behavior:?} jumps {centroid_step}");
        }
        let opaque_counts: Vec<_> = swim_frames.iter().map(collect_opaque_pixel_count).collect();
        let largest_count = *opaque_counts.iter().max().unwrap();
        // Turning toward the viewer narrows the fish, but it never vanishes.
        assert!(
            opaque_counts
                .iter()
                .all(|&opaque_count| opaque_count * 8 > largest_count)
        );
    }
}

#[test]
fn png_output_doubles_each_pixel_and_repeats_exactly() {
    let koi_development = develop_with_total(1_500);
    let koi_frame = render_koi(&koi_development);
    let png_bytes = encode_png(&koi_frame).unwrap();
    assert_eq!(
        png_bytes,
        encode_png(&render_koi(&koi_development)).unwrap()
    );
    let mut png_reader = png::Decoder::new(Cursor::new(&png_bytes))
        .read_info()
        .unwrap();
    let mut decoded_pixels = vec![0; png_reader.output_buffer_size().unwrap()];
    let frame_info = png_reader.next_frame(&mut decoded_pixels).unwrap();
    assert_eq!((frame_info.width, frame_info.height), (512, 512));
    assert_eq!(frame_info.color_type, png::ColorType::Rgba);
    for (pixel_index, native_channels) in koi_frame
        .as_rgba_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
    {
        let block_left = pixel_index as u32 % FRAME_WIDTH_PIXELS * 2;
        let block_top = pixel_index as u32 / FRAME_WIDTH_PIXELS * 2;
        for (offset_x, offset_y) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let export_index =
                ((block_top + offset_y) * frame_info.width + block_left + offset_x) as usize * 4;
            assert_eq!(
                &decoded_pixels[export_index..export_index + 4],
                native_channels
            );
        }
    }
}

#[test]
fn quiet_days_change_streak_state_without_changing_the_still_fish() {
    let mut account_snapshot: AccountSnapshot =
        serde_json::from_str(include_str!("fixtures/account-with-history.json")).unwrap();
    let active_development = derive_koi_development(&account_snapshot).unwrap();
    let contribution_history = account_snapshot.contribution_history.as_mut().unwrap();
    for day_offset in 1..=14 {
        contribution_history.days.push(ContributionDay {
            date: contribution_history
                .observed_on
                .checked_add_days(Days::new(day_offset))
                .unwrap(),
            contribution_count: 0,
        });
    }
    contribution_history.observed_on = contribution_history.days.last().unwrap().date;
    let quiet_development = derive_koi_development(&account_snapshot).unwrap();
    assert_ne!(active_development.streaks, quiet_development.streaks);
    assert_eq!(
        render_koi(&active_development),
        render_koi(&quiet_development)
    );
}

fn read_fixture_development() -> KoiDevelopment {
    let account_snapshot =
        serde_json::from_str(include_str!("fixtures/account-with-history.json")).unwrap();
    derive_koi_development(&account_snapshot).unwrap()
}

fn develop_with_total(contribution_total: u64) -> KoiDevelopment {
    let mut koi_development = read_fixture_development();
    koi_development.total_contributions = contribution_total;
    koi_development
}

fn collect_opaque_pixel_count(koi_frame: &KoiFrame) -> usize {
    koi_frame
        .as_rgba_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel_channels| pixel_channels[3] != 0)
        .count()
}

fn measure_opaque_centroid(koi_frame: &KoiFrame) -> [f64; 2] {
    let mut position_sum = [0.0; 2];
    let mut opaque_pixel_count = 0.0;
    for (pixel_index, pixel_channels) in koi_frame
        .as_rgba_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
    {
        if pixel_channels[3] != 0 {
            position_sum[0] += (pixel_index as u32 % FRAME_WIDTH_PIXELS) as f64;
            position_sum[1] += (pixel_index as u32 / FRAME_WIDTH_PIXELS) as f64;
            opaque_pixel_count += 1.0;
        }
    }
    [
        position_sum[0] / opaque_pixel_count,
        position_sum[1] / opaque_pixel_count,
    ]
}

fn collect_opaque_colors(koi_frame: &KoiFrame) -> BTreeSet<[u8; 4]> {
    koi_frame
        .as_rgba_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel_channels| pixel_channels[3] != 0)
        .copied()
        .collect()
}

struct OpaqueBounds {
    left_x: u32,
    top_y: u32,
    right_x: u32,
    bottom_y: u32,
}

fn measure_opaque_bounds(koi_frame: &KoiFrame) -> OpaqueBounds {
    let mut opaque_bounds = OpaqueBounds {
        left_x: FRAME_WIDTH_PIXELS,
        top_y: FRAME_HEIGHT_PIXELS,
        right_x: 0,
        bottom_y: 0,
    };
    for (pixel_index, pixel_channels) in koi_frame
        .as_rgba_bytes()
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
    {
        if pixel_channels[3] == 0 {
            continue;
        }
        let pixel_x = pixel_index as u32 % FRAME_WIDTH_PIXELS;
        let pixel_y = pixel_index as u32 / FRAME_WIDTH_PIXELS;
        opaque_bounds.left_x = opaque_bounds.left_x.min(pixel_x);
        opaque_bounds.top_y = opaque_bounds.top_y.min(pixel_y);
        opaque_bounds.right_x = opaque_bounds.right_x.max(pixel_x);
        opaque_bounds.bottom_y = opaque_bounds.bottom_y.max(pixel_y);
    }
    assert!(
        opaque_bounds.left_x < opaque_bounds.right_x
            && opaque_bounds.top_y < opaque_bounds.bottom_y
    );
    opaque_bounds
}
