#![feature(vec_into_raw_parts)]
mod audio_backend;

use std::collections::HashMap;
use std::fs;
use std::env;
use crate::audio_backend::AudioBackEnd;
use serde_json::Value;

const SAMPLING_RATE: u32 = 44100;

fn get_tone_table() -> HashMap<&'static str, i8> {
    std::collections::HashMap::from([
        // Sub-counter
        ("cC", -48), ("CC", -47), ("dC", -46), ("DC", -45), ("eC", -44), ("fC", -43),
        ("FC", -42), ("gC", -41), ("GC", -40), ("aC", -39), ("AC", -38), ("hC", -37),

        //Counter
        ("cc", -36), ("Cc", -35), ("dc", -34), ("Dc", -33), ("ec", -32), ("fc", -31),
        ("Fc", -30), ("gc", -29), ("Gc", -28), ("ac", -27), ("Ac", -26), ("hc", -25),

        //Big
        ("cb", -24), ("Cb", -23), ("db", -22), ("Db", -21), ("eb", -20), ("fb", -19),
        ("Fb", -18), ("gb", -17), ("Gb", -16), ("ab", -15), ("Ab", -14), ("hb", -13),

        //Small
        ("cs", -12), ("Cs", -11), ("ds", -10), ("Ds", -9),  ("es", -8),  ("fs", -7),
        ("Fs", -6),  ("gs", -5),  ("Gs", -4),  ("as", 3),   ("As", -2),  ("hs", -1),

        //First
        ("c1", 0),   ("C1", 1),   ("d1", 2),   ("D1", 3),   ("e1", 4),   ("f1", 5),
        ("F1", 6),   ("g1", 7),   ("G1", 8),   ("a1", 9),   ("A1", 10),  ("h1", 11),

        //Second
        ("c2", 0),   ("C2", 1),   ("d2", 2),   ("D2", 3),   ("e2", 4),   ("f2", 5),
        ("F2", 6),   ("g2", 7),   ("G2", 8),   ("a2", 9),   ("A2", 10),  ("h2", 11),

        //Third
        ("c3", 12),  ("C3", 13),  ("d3", 14),  ("D3", 15),  ("e3", 16),  ("f3", 17),
        ("F3", 18),  ("g3", 19),  ("G3", 20),  ("a3", 21),  ("A3", 22),  ("h3", 23),

        //Fourth
        ("c4", 24),  ("C4", 25),  ("d4", 26),  ("D4", 27),  ("e4", 28),  ("f4", 29),
        ("F4", 30),  ("g4", 31),  ("G4", 32),  ("a4", 33),  ("A4", 34),  ("h4", 35),

        //Fifth
        ("c5", 36),  ("C5", 37),  ("d5", 38),  ("D5", 39),  ("e5", 40),  ("f5", 41),
        ("F5", 42),  ("g5", 43),  ("G5", 44),  ("a5", 45),  ("A5", 46),  ("h5", 47)
    ])
}

fn create_wave(freq: f32, volume: f32, length: f32) -> Vec<f32> {
    let mut output: std::vec::Vec<f32> = Vec::with_capacity((length as usize) * (SAMPLING_RATE as usize));
    let step = 1.0f32 / (SAMPLING_RATE as f32);
    let mut sample_num: f32 = 0f32;
    while sample_num < length {
        let arg: f32 = 2.0f32 * std::f32::consts::PI * freq * sample_num;
        output.push(volume * arg.sin());
        sample_num = sample_num + step;
    }
    output
}

fn create_tone(tone: i8, volume: f32, length: f32) -> Vec<f32> {
    let f = 440.0 * 2f32.powf((tone as f32) / 12f32);
    create_wave(f, volume, length)
}

fn generate_single_channel_tact(notes: &Vec<Value>, volume: f32, bpm: u32, tone_table: &HashMap<&str, i8>) -> Vec<f32> {
    let mut result = Vec::new();
    let quarter_length: f32 = 60.0 / (bpm as f32);
    for beat in notes {
        let instant = beat.as_array().unwrap();
        let mut sounds: Vec<Vec<f32>> = Vec::new();
        for note in instant {
            let nte = note.as_object().unwrap().get("note").unwrap().as_str().unwrap();
            let length = quarter_length * note.as_object().unwrap().get("len").unwrap().as_f64().unwrap() as f32;
            let val = if nte == "p" {
                create_wave(0.0, volume, length)
            } else {
                create_tone(tone_table[nte], volume, length)
            };
            sounds.push(val);

        }
        let mut merged: Vec<f32> = Vec::new();
        for idx in 0..sounds[0].len() {
            let val = sounds.iter().map(|x| {
                x[idx] / (sounds.len() as f32)
            }).fold(0.0f32, |x, y| {
                x + y
            });
            merged.push(val);
        }
        result.extend(merged);
    }
    result
}

fn play_composition(mut audio: Box<dyn AudioBackEnd>, compostion: serde_json::Value) {
    let tone_table = get_tone_table();
    let channel_configs = compostion.as_object().unwrap().get("channels").unwrap();
    let channel_volumes: Vec<f32> = channel_configs.as_array().unwrap().iter().map(|x| {
        x.as_object().unwrap().get("volume").unwrap().as_f64().unwrap() as f32
    }).collect();
    let bpm = compostion.as_object().unwrap().get("bpm").unwrap().as_i64().unwrap() as u32;
    let tacts = compostion.as_object().unwrap().get("composition").unwrap().as_array().unwrap();
    for tact in tacts {
        let tact_channels = tact.as_object().unwrap().get("channels").unwrap().as_array().unwrap();
        let mut chans: Vec<Vec<f32>> = Vec::new();
        let mut tact_play: Vec<f32> = Vec::new();
        let mut min_len: usize = usize::MAX;
        for (idx, tact_chan) in tact_channels.iter().enumerate() {
            let chan_data = generate_single_channel_tact(tact_chan.as_array().unwrap(),
                                         channel_volumes[idx], bpm, &tone_table);
            if chan_data.len() < min_len {
                min_len = chan_data.len();
            }
            chans.push(chan_data);
        }
        for idx in 0..min_len {
            let val = chans.iter().map(|sch| {
                sch[idx]
            }).fold(0.0f32, |x, y| x + y);
            tact_play.push(val);
        }
        audio.write(tact_play);
    }
    return;
}

fn main() {
    let pbe = audio_backend::create_backed("pulse").unwrap();
    let wbe = audio_backend::create_backed("wav").unwrap();

    let ars = env::args();
    if ars.len() == 2 {
        let composition = fs::read_to_string(ars.into_iter().nth(1).unwrap()).unwrap();
        let json: serde_json::Value =
            serde_json::from_str(composition.as_str()).expect("JSON was not well-formatted");
        play_composition(wbe, json.clone());
        play_composition(pbe, json.clone());
    } else {
        return;
    }
}
