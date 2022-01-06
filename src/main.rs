use std::env;
use std::fs;
use std::collections::HashMap;
use std::str::FromStr;

// bar: 4分が4つ入る単位
// division: barを192分割して矢印があるところ
// ofs: bar中でのdivisionの位置。0から191まで

static NOTE_UNIT: i32 = 192;


#[derive(Debug)]
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
struct Arrow {
    direction: Direction,
    arrow_type: ArrowType,
    //end: f32,
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
            //end: ofs + NOTE_UNIT / 4,
            //end_time: ofs + NOTE_UNIT / 4
        });
        }
    }
    arrows
}

#[derive(Debug)]
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
            "Expert" => Ok(Difficulty::Expert),
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

fn str_to_notes(bars: Vec<&str>) -> Vec<Division> {
    let mut notes: Vec<Division> = Vec::new();
    let mut offset = 0;
    for bar in bars {
        let divisions = bar_to_divisions(bar.split("\n").filter(|&x| !x.is_empty()).collect(), offset);
        notes.extend(divisions);
        offset += NOTE_UNIT;
    }
    notes
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
    for statement in statements {
        let parts: Vec<&str> = statement.trim().split(":").collect();
        if parts.len() < 2 {
            continue;
        }
        let key = parts[0].trim_matches('#');
        let value = parts[1..].join(":");
        props.insert(key.to_string(), value.to_string());
    }

    //println!("file content:\n{:?}", props);

    let notes_content : Vec<&str> = props.get("NOTES").unwrap().split(":").collect();

    let chart = ChartInfo {
        chart_type: ChartType::from_str(notes_content[0].trim_start()).unwrap(),
        difficulty: Difficulty::from_str(notes_content[2].trim_start()).unwrap(),
        level: notes_content[3].trim_start().parse().unwrap(),
        groove_radar: notes_content[4].trim_start().split(",").map(|s| s.parse().unwrap()).collect(),
        notes: str_to_notes(notes_content[5].split(",").map(|s| s.trim_start()).collect())
    };
    println!("chart:\n{:?}", chart);

    let bpms: Vec<BPM> = props.get("BPMS").unwrap().split(",").map(|s| BPM::from_str(s.trim_end()).unwrap()).collect();
    let stops: Vec<Stop> = props.get("STOPS").unwrap().split(",").map(|s| Stop::from_str(s.trim_end()).unwrap()).collect();
    println!("BPM:\n{:?}", bpms);
    println!("stop:\n{:?}", stops);
    
    //println!("BPM:\n{:?}", props.get("BPMS"));
    //println!("stop:\n{:?}", props.get("STOPS"));
}