use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;


use crate::arrow::{Arrow, ArrowType, Division, NOTE_UNIT, bar_to_divisions, find_freeze_end};
use crate::gimmick::{Gimmick, Bpm, Stop, BpmDisplay, StopDisplay};
use crate::groove_radar::get_groove_radar;


#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum ChartType {
    DanceSingle,
    DanceDouble,
}

impl FromStr for ChartType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dance-single" => Ok(ChartType::DanceSingle),
            "dance-double" => Ok(ChartType::DanceDouble),
            _ => Err(format!("{} is not supported", s)),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Difficulty {
    Beginner,
    Easy,
    Medium,
    Hard,
    Challenge,
    Edit,
}

impl FromStr for Difficulty {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Beginner" => Ok(Difficulty::Beginner),
            "Easy" => Ok(Difficulty::Easy),
            "Medium" => Ok(Difficulty::Medium),
            "Hard" => Ok(Difficulty::Hard),
            "Challenge" => Ok(Difficulty::Challenge),
            "Edit" => Ok(Difficulty::Edit),
            _ => Err(format!("{} is not supported", s)),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct ChartInfo {
    pub chart_type: ChartType,
    pub difficulty: Difficulty,
    pub level: i32,
    pub max_combo: i32,
    //pub groove_radar: GrooveRadar,
    pub stream: i32,
    pub voltage: i32, 
    pub air: i32,
    pub freeze: i32,
    pub chaos: i32,
    //notes: Vec<Division>,
}

// TODO: contentの公開をやめて、dump的なmethodを公開すべき
pub struct Chart {
    pub info: ChartInfo,
    //notes: Vec<Division>,
    pub content: LegacyChartContent,
}

// TODO: viewerと同時に変更する
#[derive(Debug, Deserialize, Serialize)]
pub struct LegacyChartContent {
    stream: Vec<Division>,
    stream_info: Vec<i32>,
    pub gimmick: Gimmick,
}

fn str_to_notes(bars: Vec<&str>, bpms: &[Bpm], stops: &[Stop]) -> Vec<Division> {
    let mut notes: Vec<Division> = Vec::new();
    let mut offset = 0;
    for bar in bars {
        let divisions =
            bar_to_divisions(bar.split('\n').filter(|&x| !x.is_empty()).collect(), offset);
        notes.extend(divisions);
        offset += NOTE_UNIT;
    }
    // calc freeze end timing
    let mut notes_with_freeze_end: Vec<Division> = Vec::new();
    for div in &notes {
        assert_ne!(div.arrows.len(), 0);
        let mut arrows: Vec<Arrow> = Vec::new();
        for arrow in &div.arrows {
            let mut end = 0;
            if arrow.arrow_type == ArrowType::Freeze {
                end = find_freeze_end(&notes, div.offset, arrow.direction);
            }
            if arrow.arrow_type != ArrowType::FreezeEnd {
                arrows.push(Arrow {
                    direction: arrow.direction,
                    arrow_type: arrow.arrow_type,
                    end,
                    end_time: offset_to_time(end, bpms, stops),
                });
            }
        }
        // dont add if all arrows are freeze end
        if arrows.iter().any(|x| x.arrow_type != ArrowType::FreezeEnd) {
            notes_with_freeze_end.push(Division {
                arrows,
                color: div.color,
                offset: div.offset,
                time: offset_to_time(div.offset, bpms, stops),
            });
        }
    }
    notes_with_freeze_end
}


pub fn offset_to_time(offset: i32, bpms: &[Bpm], stops: &[Stop]) -> f32 {
    let mut time = 0.0;
    let mut done = 0;
    let mut prev_bpm = &bpms[0];
    for bpm in bpms {
        if bpm.offset >= offset {
            let elapsed = (offset - done) as f32 / ((NOTE_UNIT / 4) as f32);
            time += 60.0 / prev_bpm.bpm * elapsed;
            break;
        }
        let elapsed = ((bpm.offset - done) as f32) / ((NOTE_UNIT / 4) as f32);
        time += 60.0 / prev_bpm.bpm * elapsed;
        done = bpm.offset as i32;
        prev_bpm = bpm;
    }
    if bpms[bpms.len() - 1].offset < offset {
        let elapsed = (offset - done) as f32 / ((NOTE_UNIT / 4) as f32);
        time += 60.0 / bpms[bpms.len() - 1].bpm * elapsed;
    }
    for stop in stops {
        if stop.offset >= offset {
            break;
        }
        time += stop.time;
    }
    time
}
struct ChartProps {
    props: HashMap<String, String>,
    note_strings: String,
}
fn parse_chart_props(chart_string: &&str) -> ChartProps {
    let mut props = HashMap::new();
    let mut note_strings = "".to_string();
    let statements: Vec<&str> = chart_string.split(';').collect();
    for statement in statements {
        let parts: Vec<&str> = statement.trim().split(':').collect();
        if parts.len() < 2 {
            continue;
        }
        let key = parts[0].trim_matches('#');
        let value = parts[1..].join(":");

        if key == "NOTES" {
            note_strings = value.to_string();
        } else {
            props.insert(key.to_string(), value.to_string());
        }
    }
    ChartProps { props, note_strings }
}

pub fn create_chart(filepath: &String) -> Vec<Chart> {
    if filepath.ends_with(".sm") {
        sm_to_chart(filepath)
    } else if filepath.ends_with(".ssc") {
        ssc_to_chart(filepath)
    } else {
        panic!("unsupported file format");
    }
}

fn ssc_to_chart(filepath: &String) -> Vec<Chart> {
    let contents = fs::read_to_string(filepath).expect("file open error");
    // remove comment
    let statements_without_comment: Vec<&str> = contents
        .split('\n')
        .filter(|s| !s.starts_with("//"))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let contents_without_comment: String = statements_without_comment.join("\n");
    let chart_strings: Vec<&str> = contents_without_comment.split("#NOTEDATA:;").collect(); // (head, tail) = ... みたいに書きたい
    let common_statements: Vec<&str> = chart_strings[0].split(';').collect();
    let mut common_props = HashMap::new();
    let mut chart_props: Vec<ChartProps> = Vec::new();
    for statement in common_statements {
        let parts: Vec<&str> = statement.trim().split(':').collect();
        if parts.len() < 2 {
            continue;
        }
        let key = parts[0].trim_matches('#');
        let value = parts[1..].join(":");
        common_props.insert(key.to_string(), value.to_string());
    }
    for chart_string in chart_strings.iter().skip(1) {
        let chart = parse_chart_props(chart_string);
       chart_props.push(chart);
    }

    return 
       chart_props 
        .iter()
        .filter(|c| ChartType::from_str(c.props.get("STEPSTYPE").unwrap()).unwrap() == ChartType::DanceSingle)
        .map(|c| {
            let chart_type = ChartType::from_str(c.props.get("STEPSTYPE").unwrap()).unwrap();
            let difficulty = Difficulty::from_str(c.props.get("DIFFICULTY").unwrap()).unwrap();
            let bpms: Vec<Bpm> = if let Some(s) = c.props.get("BPMS") {
                s.split(',').map(|s| Bpm::from_str(s.trim_end()).unwrap()).collect()
            } else {
                common_props.get("BPMS").unwrap().split(',').map(|s| Bpm::from_str(s.trim_end()).unwrap()).collect()
            };
            // unwrap_orを使いたいけどstr周りのエラーがなんもわからん
            //let stop_str = props.get("STOPS").unwrap_or("".to_string());
            let stop_str = if let Some(s) = c.props.get("STOPS") {
                s
            } else {
                ""
            };
            let stops: Vec<Stop> = if stop_str.is_empty() {
                Vec::new()
            } else {
                stop_str
                    .split(',')
                    .map(|s| Stop::from_str(s.trim_end()).unwrap())
                    .collect()
            };
            let level = c.props.get("METER").unwrap().parse().unwrap();
            let notes = str_to_notes(
                c.note_strings.split(',').map(|s| s.trim_start()).collect(),
                &bpms,
                &stops,
            );
            let groove_radar = get_groove_radar(&notes, &bpms, &stops);
            let info = ChartInfo {
                chart_type,
                difficulty,
                level,
                max_combo: notes.len() as i32,
                stream: groove_radar.stream,
                voltage: groove_radar.voltage,
                air: groove_radar.air,
                freeze: groove_radar.freeze,
                chaos: groove_radar.chaos,
            };
            Chart {
                info,
                content: LegacyChartContent {
                    stream: notes,
                    stream_info: Vec::new(),
                    gimmick: Gimmick {
                        soflan: bpms.clone().into_iter().map(BpmDisplay::from_bpm).collect(),
                        stop: stops.clone().into_iter().map(StopDisplay::from_stop).collect(),
                    }
                },
            }
        })
        .collect();
}


fn sm_to_chart(filepath: &String) -> Vec<Chart> {
    let contents = fs::read_to_string(filepath).expect("file open error");
    // remove comment
    let statements_without_comment: Vec<&str> = contents
        .split('\n')
        .filter(|s| !s.starts_with("//"))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let contents_without_comment: String = statements_without_comment.join("\n");
    let statements = contents_without_comment.split(';');
    let mut props = HashMap::new();
    let mut notes_strings = Vec::new();
    for statement in statements {
        let parts: Vec<&str> = statement.trim().split(':').collect();
        if parts.len() < 2 {
            continue;
        }
        let key = parts[0].trim_matches('#');
        let value = parts[1..].join(":");
        if key == "NOTES" {
            notes_strings.push(value.to_string());
        } else {
            props.insert(key.to_string(), value.to_string());
        }
    }
    let bpms: Vec<Bpm> = props
        .get("BPMS")
        .unwrap()
        .split(',')
        .map(|s| Bpm::from_str(s.trim_end()).unwrap())
        .collect();
    // unwrap_orを使いたいけどstr周りのエラーがなんもわからん
    //let stop_str = props.get("STOPS").unwrap_or("".to_string());
    let stop_str = if let Some(s) = props.get("STOPS") {
        s
    } else {
        ""
    };
    let stops: Vec<Stop> = if stop_str.is_empty() {
        Vec::new()
    } else {
        stop_str
            .split(',')
            .map(|s| Stop::from_str(s.trim_end()).unwrap())
            .collect()
    };
    let notes_content: Vec<Vec<&str>> = notes_strings
        .iter()
        .map(|s| s.split(':').collect())
        .collect();

    return 
        notes_content
        .iter()
        .filter(|s| ChartType::from_str(s[0].trim_start()).unwrap() == ChartType::DanceSingle)
        .map(|s| {
            let chart_type = ChartType::from_str(s[0].trim_start()).unwrap();
            let difficulty = Difficulty::from_str(s[2].trim_start()).unwrap();
            let level = s[3].trim_start().parse().unwrap();
            let notes = str_to_notes(
                s[5].split(',').map(|s| s.trim_start()).collect(),
                &bpms,
                &stops,
            );
            let groove_radar = get_groove_radar(&notes, &bpms, &stops);
            let info = ChartInfo {
                chart_type,
                difficulty,
                level,
                max_combo: notes.len() as i32,
                stream: groove_radar.stream,
                voltage: groove_radar.voltage,
                air: groove_radar.air,
                freeze: groove_radar.freeze,
                chaos: groove_radar.chaos,
            };
            Chart {
                info,
                content: LegacyChartContent {
                    stream: notes,
                    stream_info: Vec::new(),
                    gimmick: Gimmick {
                        soflan: bpms.clone().into_iter().map(BpmDisplay::from_bpm).collect(),
                        stop: stops.clone().into_iter().map(StopDisplay::from_stop).collect(),
                    }
                },
            }
        })
        .collect();
}
