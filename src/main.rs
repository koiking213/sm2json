use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use filetime::FileTime;
use std::path::Path;
use std::str::FromStr;
use chrono::prelude::DateTime;
use std::time::{UNIX_EPOCH, Duration};

pub mod arrow;
pub mod gimmick;
pub mod chart;
pub mod groove_radar;

// bar: 4分が4つ入る単位
// division: barを192分割して矢印があるところ
// ofs: bar中でのdivisionの位置。0から191まで

#[derive(Debug, Deserialize, Serialize)]
struct Music {
    path: String,
    offset: f32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Song {
    title: String,
    dir_name: String,
    charts: Vec<chart::ChartInfo>,
    bpm: String,
    music: Music,
    banner: String,
    timestamp: String,
}

// TODO: chartは外部から受け取る
fn sm_to_song_info(dirname: String, filepath: String) -> Song {
    let contents = fs::read_to_string(&filepath).expect("file open error");
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

    // TODO: .ssc形式に対応するなら、BPM情報はChartInfoに含まれるべき
    let bpms: Vec<gimmick::Bpm> = props
        .get("BPMS")
        .unwrap()
        .split(',')
        .map(|s| gimmick::Bpm::from_str(s.trim_end()).unwrap())
        .collect();

    let charts = chart::sm_to_chart(&filepath);

    let displaybpm: String = match props.get("DISPLAYBPM") {
        Some(s) => get_disp_bpm(s),
        None => {
            let max = bpms.iter().max_by(|a, b| a.bpm.partial_cmp(&b.bpm).unwrap()).unwrap();
            let min = bpms.iter().min_by(|a, b| a.bpm.partial_cmp(&b.bpm).unwrap()).unwrap();
            if (max.bpm - min.bpm).abs() < 0.1 {
                max.bpm.round().to_string()
            } else {
                format!("{}-{}", min.bpm.round(), max.bpm.round())

            }
        }
    };
    let metadata = fs::metadata(&filepath).unwrap();
    let time = FileTime::from_last_modification_time(&metadata).seconds();
    let d = UNIX_EPOCH + Duration::from_secs(time as u64);
    let timestamp = DateTime::<chrono::Local>::from(d).format("%Y-%m-%d %H:%M:%S").to_string();
    Song {
        title: props.get("TITLE").unwrap().to_string(),
        dir_name: dirname,
        charts: charts.iter().map(|chart| chart.info).collect(),
        bpm: displaybpm,
        music: Music {
            path: props.get("MUSIC").unwrap().to_string(),
            // 良い書き方がありそう
            offset: if let Some(ofs) = props.get("OFFSET") {
                ofs.parse().unwrap()
            } else {
                0.0
            }
        },
        banner: props.get("BANNER").unwrap().to_string(),
        timestamp,
    }
}

fn get_disp_bpm(s: &str) -> String {
    let split: Vec<&str> = s.split(':').collect();
    if split.len() == 1 {
        split[0].parse::<f32>().unwrap().round().to_string()
    } else {
        let min = split[0].parse::<f32>().unwrap();
        let max = split[1].parse::<f32>().unwrap();
        format!("{}-{}", min.round(), max.round())
    }
}

// TODO: 1つの.smファイルを1つのjsonにしたほうが楽そう
fn main() {
    let args: Vec<String> = env::args().collect();
    match fs::read_dir(args[1].clone()) {
        Ok(dirs) => {
            let mut songs = Vec::new();
            for dir in dirs.into_iter().filter(|dir| dir.as_ref().unwrap().path().is_dir()) {
                let dir = dir.unwrap();
                let dirname = dir.file_name().into_string().unwrap();
                let mut files = Vec::new();
                // smファイルを探索
                match fs::read_dir(dir.path()) {
                    Ok(dirs) => {
                        for file in dirs {
                            let file = file.unwrap();
                            let filename = file.file_name().into_string().unwrap();
                            if filename.ends_with(".sm") {
                                let path = Path::new(&dir.path()).join(filename);
                                files.push(path.to_str().unwrap().to_string());
                            }
                        }
                    }
                    Err(e) => {
                        println!("failed to read_dir for {:?}: {:?}",dir, e);
                    }
                }
                // 各譜面のjsonを作りつつ曲リストに追加していく
                for file in files {
                    println!("file: {}", file);
                    let song = sm_to_song_info(dirname.clone(), file.clone());
                    let dir_path = Path::new("output").join(&dir.path());
                    fs::create_dir_all(&dir_path).unwrap();
                    let charts = chart::sm_to_chart(&file);
                    // 譜面ごとのjsonを作成
                    for chart in &charts {
                        let mut chart_path = dir_path.clone();
                        chart_path.push(format!("{:?}.json", chart.info.difficulty));
                        println!("{:?}", chart_path);
                        let chart_json = serde_json::to_string(&chart.content).unwrap();
                        fs::write(chart_path, chart_json).unwrap();
                    }

                    // 曲リスト更新
                    songs.push(song);
                }
            }
            let j = serde_json::to_string(&songs).unwrap();
            fs::write(Path::new("output").join("songs.json"), j).unwrap();
        }
        Err(e) => {
            println!("failed to open root directory: {:?}", e);
        }
    }
}
