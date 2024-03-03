# sm2json

StepManiaの譜面ファイル(.sm, .ssc)をJSONに変換する。

## 準備

TBD

## 使い方

TBD

## 出力形式

各譜面に対して1つのJSONが生成され、それとは別に全曲リストのJSONが生成される。

### 曲リスト(`songs.json`)

```
[{
    title: "曲のタイトル",
    dir_name: "曲が格納されているディレクトリのパス(コマンド引数に与えたディレクトリからの相対パス)",
    charts: [
        chart_type: ChartType,
        difficulty: "Beginner, Easy, Medium, Hard, Challenge, Edit" のいずれか,
        level: "難度値",
        max_combo: "最大コンボ数",
        stream: グルーブレーダーのstream,
        voltage: グルーブレーダーのvoltage, 
        air: グルーブレーダーのair,
        freeze: グルーブレーダーのfreeze,
        chaos: グルーブレーダーのchaos,
    ],
    bpm: "表記BPM",
    music: {
        path: 音声ファイルのパス (譜面ファイルからの相対パス),
        offset: 曲のオフセット (譜面ファイルのOFFSETの値で、例えばArrowVortexだとADJUST SYNCのMusic offsetに相当)
    },
    banner: "バナー画像のパス(譜面ファイルからの相対パス)",
    timestamp: "譜面ファイルの更新日時",
}]
```

### 各譜面

TBD
