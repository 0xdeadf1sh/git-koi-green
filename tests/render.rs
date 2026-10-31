use std::collections::BTreeSet;
use std::io::Cursor;

use chrono::Days;
use git_koi::behavior::Behavior;
use git_koi::history::{AccountSnapshot, ContributionDay};
use git_koi::koi::{KoiDevelopment, derive_koi_development};
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
/// The fish's lightest green, which bubble glints, vine leaves, and sails use.
const LIGHT_GREEN: [u8; 4] = [132, 217, 101, 255];

#[test]
fn languages_and_accounts_do_not_change_the_picture() {
    let koi_development = develop_with_total(1_500);
    let mut other_development = develop_with_total(1_500);
    other_development.language_shares.clear();
    other_development.language_colors.clear();
    other_development.identity_seed += 1;
    assert_eq!(render_koi(&koi_development), render_koi(&other_development));
}

#[test]
fn every_loop_closes_seamlessly() {
    let koi_development = develop_with_total(1_500);
    for behavior in BEHAVIORS {
        let loop_frame_count = frame_count(behavior);
        assert_eq!(
            render_swim_frame(&koi_development, behavior, loop_frame_count),
            render_swim_frame(&koi_development, behavior, 0),
            "{behavior:?}"
        );
    }
}

#[test]
fn the_tank_fills_the_ground_and_the_vines_sway() {
    let koi_development = develop_with_total(1_500);
    let still_frame = render_koi(&koi_development);
    let frame_pixels = pixels_of(&still_frame);
    let bottom_row = &frame_pixels[((FRAME_HEIGHT_PIXELS - 1) * FRAME_WIDTH_PIXELS) as usize..];
    let covered_share =
        bottom_row.iter().filter(|pixel| pixel[3] != 0).count() as f64 / bottom_row.len() as f64;
    assert!(covered_share > 0.85, "{covered_share}");
    let sway_frames: Vec<_> = [0, 30, 60]
        .map(|frame_index| render_swim_frame(&koi_development, Behavior::DeepRest, frame_index))
        .into_iter()
        .collect();
    // Below the fish's water, only the swaying vines change from frame to frame.
    let ground_start = ((FRAME_HEIGHT_PIXELS - 46) * FRAME_WIDTH_PIXELS) as usize;
    assert_ne!(
        pixels_of(&sway_frames[0])[ground_start..],
        pixels_of(&sway_frames[1])[ground_start..]
    );
    assert_ne!(
        pixels_of(&sway_frames[1])[ground_start..],
        pixels_of(&sway_frames[2])[ground_start..]
    );
}

#[test]
fn bubbles_rise_through_the_water_above_the_fish() {
    let koi_development = develop_with_total(1_500);
    let top_band_colors = |koi_frame: &KoiFrame| {
        pixels_of(koi_frame)[..(24 * FRAME_WIDTH_PIXELS) as usize]
            .iter()
            .filter(|pixel| pixel[3] != 0)
            .copied()
            .collect::<BTreeSet<_>>()
    };
    let rest_colors: BTreeSet<_> = (0..frame_count(Behavior::Rest))
        .step_by(5)
        .flat_map(|frame_index| {
            top_band_colors(&render_swim_frame(
                &koi_development,
                Behavior::Rest,
                frame_index,
            ))
        })
        .collect();
    assert!(rest_colors.contains(&LIGHT_GREEN));
}

#[test]
fn surface_feeding_draws_pellets_that_rest_does_not() {
    const PELLET_COLOR: [u8; 4] = [228, 166, 114, 255];
    let koi_development = develop_with_total(1_500);
    let colors_of = |behavior| {
        (0..frame_count(behavior))
            .step_by(5)
            .flat_map(|frame_index| {
                pixels_of(&render_swim_frame(&koi_development, behavior, frame_index)).to_vec()
            })
            .collect::<BTreeSet<_>>()
    };
    assert!(colors_of(Behavior::SurfaceFeeding).contains(&PELLET_COLOR));
    assert!(!colors_of(Behavior::Rest).contains(&PELLET_COLOR));
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
    assert_eq!(
        (frame_info.width, frame_info.height),
        (FRAME_WIDTH_PIXELS * 2, FRAME_HEIGHT_PIXELS * 2)
    );
    assert_eq!(frame_info.color_type, png::ColorType::Rgba);
    for (pixel_index, native_channels) in pixels_of(&koi_frame).iter().enumerate() {
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
fn quiet_days_change_streak_state_without_changing_the_still_picture() {
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

fn develop_with_total(contribution_total: u64) -> KoiDevelopment {
    let account_snapshot =
        serde_json::from_str(include_str!("fixtures/account-with-history.json")).unwrap();
    let mut koi_development = derive_koi_development(&account_snapshot).unwrap();
    koi_development.total_contributions = contribution_total;
    koi_development
}

fn pixels_of(koi_frame: &KoiFrame) -> &[[u8; 4]] {
    koi_frame.as_rgba_bytes().as_chunks::<4>().0
}
