use serde::{Deserialize, Serialize};
use std::str::FromStr;

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