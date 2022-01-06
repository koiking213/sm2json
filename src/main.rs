use std::env;
use std::fs;
use std::collections::HashMap;
use std::str::FromStr;

// bar: 4分が4つ入る単位
// division: barを192分割して矢印があるところ
// ofs: bar中でのdivisionの位置。0から191まで

static NOTE_UNIT: i32 = 192;


#[derive(Debug, Clone, Copy)]
enum Color {
    Red,
    Blue,
    Yellow,
    Green,
}

fn ofs_to_color(ofs: i32) -> Color {
    if ofs % (NOTE_UNIT / 4) == 0 {
        Color::Red
    } else if ofs % (NOTE_UNIT / 8) == 0 {
        Color::Blue
    } else if ofs % (NOTE_UNIT / 16) == 0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn int_to_direction(i: i32) -> Direction {
    match i {
        0 => Direction::Left,
        1 => Direction::Down,
        2 => Direction::Up,
        3 => Direction::Right,
        _ => panic!("invalid direction"),
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ArrowType {
    None, 
    Normal,
    Freeze,
    FreezeEnd,
    Mine
}

impl FromStr for ArrowType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(ArrowType::None),
            "1" => Ok(ArrowType::Normal),
            "2" => Ok(ArrowType::Freeze),
            "3" => Ok(ArrowType::FreezeEnd),
            "M" => Ok(ArrowType::Mine),
            _ => Err(format!("{} is not arrow type", s))
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
struct Arrow {
    direction: Direction,
    arrow_type: ArrowType,
    end: i32,
    //end_time: f32
}

// "0012" -> [Arrow(Up, Normal), Arrow(Right, Freeze)]
fn make_arrows(s: &str) -> Vec<Arrow> {
    let mut arrows: Vec<Arrow> = Vec::new();
    if s.len() != 4 {
        panic!("{} is not 4 length", s);
    }
    for (i, c) in s.chars().enumerate() {
        //let ofs = i * (NOTE_UNIT / 4);
        let arrow_type = ArrowType::from_str(&c.to_string()).unwrap();
        if arrow_type != ArrowType::None {
        arrows.push(Arrow {
            direction: int_to_direction(i as i32),
            arrow_type: arrow_type,
            end: 0
            //end_time: ofs + NOTE_UNIT / 4
        });
        }
    }
    arrows
}

#[derive(Debug, Clone)]
struct Division {
    arrows: Vec<Arrow>,
    color: Color,
    offset: i32,
    //time: f32,
}

fn bar_to_divisions(bar: Vec<&str>, offset: i32) -> Vec<Division> {
    let mut divisions: Vec<Division> = Vec::new();
    if NOTE_UNIT % (bar.len() as i32) != 0 {
        panic!("{:?} is not a valid var", bar);
    }
    let epsilon = NOTE_UNIT / (bar.len() as i32);
    for (i, division) in bar.iter().enumerate() {
        let ofs_in_bar = i as i32 * epsilon;
        let color = ofs_to_color(ofs_in_bar);
        let arrows = make_arrows(division);
        if !arrows.is_empty() {
            divisions.push(Division {
                arrows: arrows,
                color: color,
                offset: offset + ofs_in_bar,
                //time: offset + NOTE_UNIT / 4
            });
        }
    }
    divisions
}

#[derive(Debug)]
enum ChartType {
    DanceSingle,
    DanceDouble,
}

impl FromStr for ChartType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dance-single" => Ok(ChartType::DanceSingle),
            "dance-double" => Ok(ChartType::DanceDouble),
            _ => Err(format!("{} is not supported", s))
        }
    }
}

#[derive(Debug)]
enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
    Edit,
}

impl FromStr for Difficulty{
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Easy" => Ok(Difficulty::Easy),
            "Medium" => Ok(Difficulty::Medium),
            "Hard" => Ok(Difficulty::Hard),
            "Challenge" => Ok(Difficulty::Expert),
            "Edit" => Ok(Difficulty::Edit),
            _ => Err(format!("{} is not supported", s))
        }
    }
}

#[derive(Debug)]
struct ChartInfo {
    chart_type: ChartType,
    difficulty: Difficulty,
    level: i32,
    groove_radar: Vec<i32>,
    //notes: Vec<String>
    notes: Vec<Division>,
}

fn find_freeze_end(notes: &Vec<Division>, offset: i32, direction: Direction) -> i32 {
    for division in notes {
        if division.offset <= offset {
            continue;
        }
        for arrow in &division.arrows {
            if arrow.direction == direction && arrow.arrow_type == ArrowType::FreezeEnd {
                return division.offset;
            }
        }
    }
    panic!("no freeze end found");
}

fn str_to_notes(bars: Vec<&str>) -> Vec<Division> {
    let mut notes: Vec<Division> = Vec::new();
    let mut offset = 0;
    for bar in bars {
        let divisions = bar_to_divisions(bar.split("\n").filter(|&x| !x.is_empty()).collect(), offset);
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
            arrows.push(Arrow {
                direction: arrow.direction,
                arrow_type: arrow.arrow_type,
                end: end
            });
        }
        notes_with_freeze_end.push(Division {
            arrows: arrows,
            color: div.color,
            offset: div.offset,
        });
    }
    notes_with_freeze_end
}

#[derive(Debug)]
struct Stop {
    bar: f32,
    time: f32,
}
impl FromStr for Stop {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split("=");
        let bar = s.next().unwrap().parse::<f32>().unwrap();
        let time = s.next().unwrap().parse::<f32>().unwrap();
        Ok(Stop {
            bar: bar,
            time: time,
        })
    }
}

#[derive(Debug)]
struct BPM {
    bar: f32,
    bpm: f32,
}
impl FromStr for BPM {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split("=");
        let bar = s.next().unwrap().parse::<f32>().unwrap();
        let bpm = s.next().unwrap().parse::<f32>().unwrap();
        Ok(BPM{
            bar: bar,
            bpm: bpm,
        })
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let contents = fs::read_to_string(filename)
    .expect("file open error");
    // remove comment
    let statements_without_comment: Vec<&str> = contents.split("\n").filter(|s| !s.starts_with("//")).map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    let contents_without_comment: String = statements_without_comment.join("\n");
    let statements = contents_without_comment.split(";");
    let mut props = HashMap::new();
    let mut notes_strings = Vec::new();
    for statement in statements {
        let parts: Vec<&str> = statement.trim().split(":").collect();
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

    //println!("file content:\n{:?}", props);

    let bpms: Vec<BPM> = props.get("BPMS").unwrap().split(",").map(|s| BPM::from_str(s.trim_end()).unwrap()).collect();
    let stops: Vec<Stop> = props.get("STOPS").unwrap().split(",").map(|s| Stop::from_str(s.trim_end()).unwrap()).collect();
    println!("BPM:\n{:?}", bpms);
    println!("stop:\n{:?}", stops);

    let notes_content : Vec<Vec<&str>> = notes_strings.iter().map(|s| s.split(":").collect()).collect();

    let charts : Vec<ChartInfo> = notes_content.iter().map(|s| {
        let chart_type = ChartType::from_str(s[0].trim_start()).unwrap();
        let difficulty = Difficulty::from_str(s[2].trim_start()).unwrap();
        let level = s[3].trim_start().parse().unwrap();
        let groove_radar = s[4].trim_start().split(",").map(|s| s.parse().unwrap()).collect();
        let notes = str_to_notes(s[5].split(",").map(|s| s.trim_start()).collect());
        ChartInfo {
            chart_type: chart_type,
            difficulty: difficulty,
            level: level,
            groove_radar: groove_radar,
            notes: notes,
        }
    }).collect();
    println!("charts:\n{:?}", charts);


    // fill end of freeze arrow
    
    //charts.iter_mut().map(|chart| {
    //    let mut notes = chart.notes;
    //});
}