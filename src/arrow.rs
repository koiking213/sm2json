use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub const NOTE_UNIT: i32 = 192;

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
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

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ArrowType {
    None,
    Normal,
    Freeze,
    FreezeEnd,
    Mine,
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
            _ => Err(format!("{} is not arrow type", s)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Arrow {
    pub direction: Direction,
    pub arrow_type: ArrowType,
    pub end: i32,
    pub end_time: f32,
}

impl Arrow {
    pub fn is_freeze_end(&self, direction: Direction) -> bool {
        self.arrow_type == ArrowType::FreezeEnd && self.direction == direction
    }
}

// "0012" -> [Arrow(Up, Normal), Arrow(Right, Freeze)]
pub fn make_arrows(s: &str) -> Vec<Arrow> {
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
                arrow_type,
                end: 0,
                end_time: 0.0,
            });
        }
    }
    arrows
}

#[test]
fn test_make_arrows() {
    assert_eq!(
        make_arrows("0012"),
        vec![
            Arrow {
                direction: Direction::Up,
                arrow_type: ArrowType::Normal,
                end: 0,
                end_time: 0.0,
            },
            Arrow {
                direction: Direction::Right,
                arrow_type: ArrowType::Freeze,
                end: 0,
                end_time: 0.0,
            },
        ]
    );
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
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


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Division {
    pub arrows: Vec<Arrow>,
    pub color: Color,
    pub offset: i32,
    pub time: f32,
}

impl Division {
    pub fn is_jump (&self) -> bool {
    self.arrows.iter().filter(|a| a.arrow_type == ArrowType::Freeze || a.arrow_type == ArrowType::Normal).count() == 2
    }
    pub fn is_shock (&self) -> bool {
        self.arrows.iter().any(|a| a.arrow_type == ArrowType::Mine)
    }
}

pub fn bar_to_divisions(bar: Vec<&str>, offset: i32) -> Vec<Division> {
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
                arrows,
                color,
                offset: offset + ofs_in_bar,
                time: 0.0,
            });
        }
    }
    divisions
}

pub fn find_freeze_end(notes: &[Division], offset: i32, direction: Direction) -> i32 {
    for division in notes {
        if division.offset <= offset {
            continue;
        }
        for arrow in &division.arrows {
            if arrow.is_freeze_end(direction) {
                return division.offset;
            }
        }
    }
    panic!("no freeze end found");
}
