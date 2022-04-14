use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::str::FromStr;
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
    bpm: f32,
    music: Music,
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

    let (charts, _) = chart::sm_to_chart(filepath);

    let displaybpm: f32 = match props.get("DISPLAYBPM") {
        Some(s) => get_max_disp_bpm(s),
        None => match bpms
            .iter()
            .max_by(|a, b| a.bpm.partial_cmp(&b.bpm).unwrap())
        {
            Some(bpm) => bpm.bpm,
            None => unreachable!(),
        },
    };
    return Song {
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
    };
}

fn get_max_disp_bpm(s: &str) -> f32 {
    let split: Vec<&str> = s.split(':').collect();
    split.last().unwrap().parse().unwrap()
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
                    let (charts, gimmick) = chart::sm_to_chart(file.clone());
                    // 譜面ごとのjsonを作成
                    for chart in &charts {
                        let mut chart_path = dir_path.clone();
                        chart_path.push(format!("{:?}.json", chart.info.difficulty));
                        println!("{:?}", chart_path);
                        let chart_json = serde_json::to_string(&chart.content).unwrap();
                        fs::write(chart_path, chart_json).unwrap();
                    }
                    // 曲のギミック情報を作成 (TODO: .ssc対応時にはギミックは譜面ごとに作成する)
                    fs::write(dir_path.join("gimmick.json"), serde_json::to_string(&gimmick).unwrap()).unwrap();

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
