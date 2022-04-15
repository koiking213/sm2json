use serde::{Deserialize, Serialize};
const NOTE_UNIT: i32 = 192;

use crate::arrow::{Division, Color};
use itertools::Itertools;
use crate::gimmick::{Bpm, Stop};
use crate::chart::offset_to_time;

// TODO: 曲の長さの定義を決める

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct GrooveRadar {
    pub stream : i32,
    pub voltage: i32,
    pub air: i32,
    pub freeze: i32,
    pub chaos: i32,
}

// freezeを考慮しないといけない気がする
fn get_music_length(notes: &[Division]) -> f32 {
    //notes.last().unwrap().time - notes.first().unwrap().time + 9.0
    let last_note = notes.last().unwrap();
    let end = last_note.arrows.iter().filter(|a| a.is_freeze()).map(|a| a.end_time).fold(last_note.time, |m, v| m.max(v));
    //end - notes.first().unwrap().time + 9.0
    end + 1.6
}

fn count_subsequent_notes(notes: &[Division], offset: i32) -> i32 {
    let mut count = 0;
    for division in notes {
        if division.offset <= offset {
            continue;
        }
        count += 1;
        if division.offset >= offset + NOTE_UNIT {
            break;
        }
    }
    count
}

// bpm_offsets: 20, 30, 34
// divisions: 1, 4, 10, 12, 21, 30, 35
//   -> [1,4,10,12], [21,30], [], [35]
fn create_bpm_section_list(notes: &[Division], bpms: &[Bpm]) -> Vec<Vec<Division>> {
    let mut partitions: Vec<i32> = bpms.iter().map(|bpm| bpm.offset).collect();
    partitions.push(notes.last().unwrap().offset);
    let ranges = partitions.windows(2).map(|pair| (pair[0], pair[1]));
    ranges.map(|range| 
        notes.iter().filter(|div| div.offset >= range.0 && div.offset < range.1).cloned().collect()
    ).collect()
}

fn calc_max_notes_in_bpm_section(section: &[Division]) -> i32 {
    if section.is_empty() {
        0
    } else {
        section.iter().map(|d| count_subsequent_notes(section, d.offset)).max().unwrap()
    }
}

fn calc_max_note_density(notes: &[Division], bpms: &[Bpm]) -> i32 {
    let bpm_section_list = create_bpm_section_list(notes, bpms);
    bpm_section_list.iter().map(|s| calc_max_notes_in_bpm_section(s)).max().unwrap()
}

fn calc_stream(notes: &[Division]) -> i32 {
    let notes_per_min = (notes.len() as f32 / get_music_length(notes)) * 60.0;
    if notes_per_min < 300.0 {
        (notes_per_min / 3.0) as i32
    } else {
        ((notes_per_min - 139.0) * 100.0 / 161.0) as i32
    }
}

fn calc_beat_count(notes: &[Division], bpms: &[Bpm], stops: &[Stop]) -> f32 {
    let end = Bpm {offset: notes.last().unwrap().offset, bpm:0.0};
    let bpms_with_end = bpms.iter().chain(std::iter::once(&end));
    let mut num_beats = 0.0;
    // 停止の扱いが不明
    for (current_bpm, next_bpm) in bpms_with_end.tuple_windows() {
        let start = offset_to_time(current_bpm.offset, bpms, stops);
        let end = offset_to_time(next_bpm.offset, bpms, stops);
        num_beats += (end - start) * current_bpm.bpm;
    }
    num_beats / 60.0
}

fn calc_average_bpm(notes: &[Division], bpms: &[Bpm], stops: &[Stop]) -> f32 {
    calc_beat_count(notes, bpms, stops) * 60.0 / get_music_length(notes)
}

fn calc_voltage(notes: &[Division], bpms: &[Bpm], stops: &[Stop]) -> i32{
    let max_density = calc_max_note_density(notes, bpms);
    let average_bpm = calc_average_bpm(notes, bpms, stops);
    let max_density_per_min = (max_density as f32) * average_bpm / 4.0;
    if max_density_per_min < 600.0 {
        (max_density_per_min / 6.0) as i32
    } else {
        ((max_density_per_min + 594.0) * 100.0 / 1194.0) as i32
    }
}

fn calc_air(notes: &[Division]) -> i32{
    let jumps = notes.iter().filter(|d| d.is_jump()).count();
    let shocks = notes.iter().filter(|d| d.is_shock()).count();
    let jump_per_min = ((jumps + shocks) * 60) as f32 / get_music_length(notes);
    if jump_per_min < 55.0 {
        (jump_per_min * 20.0 / 11.0) as i32
    } else {
        ((jump_per_min + 36.0) * 100.0 / 91.0) as i32
    }
}

fn calc_freeze(notes: &[Division], bpms: &[Bpm], stops: &[Stop]) -> i32{
    let total_len: i32 = notes.iter().map(|d| {
        // 良い書き方がありそう
        let len = d.arrows.iter().filter(|a| a.is_freeze()).map(|a| a.end - d.offset).max();
        len.unwrap_or(0)
    }).sum::<i32>() / (NOTE_UNIT/4);
    let freeze_ratio = (10000 * total_len) as f32 / calc_beat_count(notes, bpms, stops);
    if freeze_ratio < 3500.0 {
        (freeze_ratio / 35.0) as i32
    } else {
        ((freeze_ratio + 2484.0) * 100.0 / 5984.0) as i32
    }
}

fn color_weight(color: &Color) -> f32 { 
    match color {
        Color::Red => 0.0,
        Color::Blue => 2.0,
        Color::Yellow=> 4.0,
        Color::Green => 5.0,
    }
}

fn calc_chaos_base_value(notes: &[Division]) -> f32 {
    let mut base_value = 0.0;
    for (prev_note, current_note) in notes.iter().tuple_windows() {
        let interval = current_note.offset - prev_note.offset;
        let inc = (current_note.arrows.len() as f32) * color_weight(&current_note.color) * ((NOTE_UNIT / 4) as f32 / interval as f32);
        base_value += inc;
    }
    base_value
}

fn calc_total_bpm_change(bpms: &[Bpm], stops: &[Stop]) -> f32 {
    // 
    #[derive(Debug)]
    enum Kind {
        Bpm,
        Stop,
    }
    // TODO: enumできれいに書けるかも
    #[derive(Debug)]
    struct BpmOrStop {
        offset: i32,
        value: f32,
        kind: Kind,
    }
    let mut gimmicks: Vec<BpmOrStop> = Vec::new();
    for bpm in bpms {
        gimmicks.push(BpmOrStop {offset: bpm.offset, value: bpm.bpm, kind: Kind::Bpm});
    }
    for stop in stops {
        gimmicks.push(BpmOrStop {offset: stop.offset, value: stop.time, kind: Kind::Stop});
    }
    // TODO: stopとbpmが同じタイミングで起きる場合はstopのみ考慮する
    gimmicks.sort_by(|a,b| a.offset.cmp(&b.offset));
    let mut total_bpm_change = 0.0;
    let mut current_bpm = bpms[0].bpm;
    for gimmick in gimmicks {
        match gimmick.kind {
            Kind::Bpm => {
                total_bpm_change += (gimmick.value - current_bpm).abs();
                current_bpm = gimmick.value;
            },
            Kind::Stop => {
                total_bpm_change += current_bpm;
            }
        }
    }
    total_bpm_change
}

fn calc_chaos(notes: &[Division], bpms: &[Bpm], stops: &[Stop]) -> i32{
    let music_length = get_music_length(notes);
    let base_value = calc_chaos_base_value(notes);
    let change_per_min = calc_total_bpm_change(bpms, stops) * 60.0 / music_length;
    let change_correction = 1.0 + (change_per_min / 1500.0);
    let chaos_degree = base_value * change_correction * 100.0 / music_length;
    if chaos_degree < 2000.0 {
        (chaos_degree / 20.0) as i32
    } else {
        ((chaos_degree + 21605.0) * 100.0 / 23605.0) as i32
    }

 
}

pub fn get_groove_radar(notes: &[Division], bpms:&[Bpm], stops: &[Stop]) -> GrooveRadar {
    GrooveRadar {
        stream: calc_stream(notes),
        voltage: calc_voltage(notes, bpms, stops),
        air: calc_air(notes),
        freeze: calc_freeze(notes, bpms, stops),
        chaos: calc_chaos(notes, bpms, stops),
    }
}