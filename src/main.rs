use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::str::FromStr;

mod chart;

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

fn sm_to_song_info(dirname: String, filepath: String) -> Song {
    let contents = fs::read_to_string(filepath).expect("file open error");
    // remove comment
    let statements_without_comment: Vec<&str> = contents
        .split("\n")
        .filter(|s| !s.starts_with("//"))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
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

    // TODO: .ssc形式に対応するなら、BPM情報はChartInfoに含まれるべき
    let bpms: Vec<chart::BPM> = props
        .get("BPMS")
        .unwrap()
        .split(",")
        .map(|s| chart::BPM::from_str(s.trim_end()).unwrap())
        .collect();

    let notes_content: Vec<Vec<&str>> = notes_strings
        .iter()
        .map(|s| s.split(":").collect())
        .collect();

    // TODO: chart側でコンストラクタを用意する
    let charts: Vec<chart::ChartInfo> = notes_content
        .iter()
        .map(|s| {
            let chart_type = chart::ChartType::from_str(s[0].trim_start()).unwrap();
            let difficulty = chart::Difficulty::from_str(s[2].trim_start()).unwrap();
            let level = s[3].trim_start().parse().unwrap();
            let groove_radar = s[4]
                .trim_start()
                .split(",")
                .map(|s| s.parse().unwrap())
                .collect();
            chart::ChartInfo {
                chart_type,
                difficulty,
                level,
                groove_radar,
            }
        })
        .collect();

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
        charts,
        bpm: displaybpm,
        music: Music {
            path: props.get("MUSIC").unwrap().to_string(),
            offset: props.get("OFFSET").unwrap().parse().unwrap(),
        },
    };
}

fn get_max_disp_bpm(s: &str) -> f32 {
    let split: Vec<&str> = s.split(":").collect();
    return split[1].parse().unwrap();
}

// TODO: 1つの.smファイルを1つのjsonにしたほうが楽そう
fn main() {
    let args: Vec<String> = env::args().collect();
    match fs::read_dir(args[1].clone()) {
        Ok(dirs) => {
            let mut songs = Vec::new();
            for dir in dirs {
                let dir = dir.unwrap();
                let dirname = dir.file_name().into_string().unwrap();
                let mut files = Vec::new();
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
                        println!("{:?}", e);
                    }
                }
                for file in files {
                    let song = sm_to_song_info(dirname.clone(), file.clone());
                    let dir_path = Path::new("output").join(&dir.path());
                    fs::create_dir_all(&dir_path).unwrap();
                    let charts = chart::sm_to_chart(file.clone());
                    for chart in &charts {
                        let mut chart_path = dir_path.clone();
                        chart_path.push(format!("{:?}.json", chart.info.difficulty));
                        println!("{:?}", chart_path);
                        let chart_json = serde_json::to_string(&chart.content).unwrap();
                        fs::write(chart_path, chart_json).unwrap();
                    }
                    songs.push(song);
                    //let mut file = File::create(outdir)?;
                }
            }
            let j = serde_json::to_string(&songs).unwrap();
            println!("{}", j);
        }
        Err(e) => {
            println!("{:?}", e);
        }
    }
}
