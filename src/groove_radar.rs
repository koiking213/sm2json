use serde::{Deserialize, Serialize};
const NOTE_UNIT: i32 = 192;

use crate::arrow::Division;
use itertools::Itertools;
use crate::gimmick::{Bpm, Stop};
use crate::chart::{offset_to_time};

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct GrooveRadar {
    pub stream : i32,
    pub voltage: i32,
    pub air: i32,
    pub freeze: i32,
    pub chaos: i32,
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
    return count;
}

// bpm_offsets: 20, 30, 34
// divisions: 1, 4, 10, 12, 21, 30, 35
//   -> [1,4,10,12], [21,30], [], [35]
fn create_bpm_section_list(notes: &[Division], bpms: &[Bpm]) -> Vec<Vec<Division>> {
    let mut partitions: Vec<i32> = bpms.iter().map(|bpm| bpm.offset).collect();
    partitions.push(notes.last().unwrap().offset);
    let ranges = partitions.windows(2).map(|pair| (pair[0], pair[1]));
    return ranges.map(|range| 
        notes.iter().filter(|div| div.offset >= range.0 && div.offset < range.1).cloned().collect()
    ).collect();
}

fn calc_max_notes_in_bpm_section(section: &[Division]) -> i32 {
    return if section.is_empty() {
        0
    } else {
        section.iter().map(|d| count_subsequent_notes(section, d.offset)).max().unwrap()
    };
}

fn calc_max_note_density(notes: &[Division], bpms: &[Bpm]) -> i32 {
    let bpm_section_list = create_bpm_section_list(notes, bpms);
    return bpm_section_list.iter().map(|s| calc_max_notes_in_bpm_section(s)).max().unwrap();
}

fn calc_stream(notes: &[Division]) -> i32 {
    let length = notes.last().unwrap().time - notes[0].time;
    let notes_per_min = (notes.len() as f32 / length) * 60.0;
    if notes_per_min < 300.0 {
        return (notes_per_min / 3.0) as i32;
    } else {
        return ((notes_per_min - 139.0) * 100.0 / 161.0) as i32;
    }
}

fn calc_average_bpm(notes: &[Division], bpms: &[Bpm], stops: &[Stop]) -> f32 {
    let music_length = notes.last().unwrap().time;
    let end = Bpm {offset: notes.last().unwrap().offset, bpm:0.0};
    let bpms_with_end = bpms.iter().chain(std::iter::once(&end));
    let mut num_beats = 0.0;
    // 停止の扱いが不明
    for (current_bpm, next_bpm) in bpms_with_end.tuple_windows() {
        let start = offset_to_time(current_bpm.offset, bpms, stops);
        let end = offset_to_time(next_bpm.offset, bpms, stops);
        num_beats += (end - start) * current_bpm.bpm;
    }
    return num_beats / music_length;
}

fn calc_voltage(notes: &[Division], bpms: &[Bpm], stops: &[Stop]) -> i32{
    let max_density = calc_max_note_density(notes, bpms);
    let average_bpm = calc_average_bpm(notes, bpms, stops);
    let max_density_per_min = (max_density as f32) * average_bpm / 4.0;
    if max_density_per_min < 600.0 {
        return (max_density_per_min / 6.0) as i32;
    } else {
        return ((max_density_per_min + 594.0) * 100.0 / 1194.0) as i32;
    }

}

fn calc_air(notes: &[Division]) -> i32{
    let jumps = notes.iter().filter(|d| d.is_jump()).count();
    let shocks = notes.iter().filter(|d| d.is_shock()).count();
    let music_length = notes.last().unwrap().time;
    let jump_per_min = ((jumps + shocks) as f32 / music_length) * 60.0;
    return if jump_per_min < 55.0 {
        (jump_per_min * 20.0 / 11.0) as i32
    } else {
        ((jump_per_min - 1.0) * 50.0 / 27.0) as i32
    }
}

fn calc_freeze(notes: &[Division]) -> i32{
0
}

fn calc_chaos(notes: &[Division]) -> i32{
0
}

pub fn get_groove_radar(notes: &[Division], bpms:&[Bpm], stops: &[Stop]) -> GrooveRadar {
    GrooveRadar {
        stream: calc_stream(notes),
        voltage: calc_voltage(notes, bpms, stops),
        air: calc_air(notes),
        freeze: calc_freeze(notes),
        chaos: calc_chaos(notes),
    }
}