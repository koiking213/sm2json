use std::str::FromStr;
use serde::{Deserialize, Serialize};
const NOTE_UNIT: i32 = 192;

#[derive(Debug, Deserialize, Serialize)]
pub struct Stop {
    pub offset: i32,
    pub time: f32,
}
impl FromStr for Stop {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split('=');
        let offset = (s.next().unwrap().parse::<f32>().unwrap() * (NOTE_UNIT / 4) as f32) as i32;
        let time = s.next().unwrap().parse::<f32>().unwrap();
        Ok(Stop { offset, time })
    }
}

// TODO: bpmの公開をやめてmaxを提供する
#[derive(Debug, Deserialize, Serialize)]
pub struct Bpm {
    pub offset: i32,
    pub bpm: f32,
}
impl FromStr for Bpm {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split('=');
        let offset = (s.next().unwrap().parse::<f32>().unwrap() * (NOTE_UNIT / 4) as f32) as i32;
        let bpm = s.next().unwrap().parse::<f32>().unwrap();
        Ok(Bpm { offset, bpm })
    }
}

// TODO: viewer側でdivisionではなくoffsetを取るようにする
#[derive(Debug, Deserialize, Serialize)]
pub struct BpmDisplay {
    pub division: f32,
    pub bpm: f32,
}
impl BpmDisplay {
    pub fn from_bpm(bpm: Bpm) -> Self {
        BpmDisplay {
            division: (bpm.offset as f32) / NOTE_UNIT as f32,
            bpm: bpm.bpm,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StopDisplay {
    pub division: f32,
    pub time: f32,
}
impl StopDisplay {
    pub fn from_stop(stop: Stop) -> Self {
        StopDisplay {
            division: (stop.offset as f32) / NOTE_UNIT as f32,
            time: stop.time,
        }
    }
}



#[derive(Debug, Deserialize, Serialize)]
pub struct Gimmick {
    pub soflan: Vec<BpmDisplay>,
    pub stop: Vec<StopDisplay>,
}