// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright © 2021 Adrian <adrian.eddy at gmail>

use std::time::Instant;
use argh::FromArgs;
use std::sync::{ Arc, atomic::AtomicBool };

use telemetry_parser::*;
use telemetry_parser::tags_impl::*;

/** gyro2bb v0.2.8
Author: Adrian <adrian.eddy@gmail.com>

Extract gyro data from Sony, GoPro and Insta360 cameras to betaflight blackbox csv log
*/
#[derive(FromArgs)]
struct Opts {
    /// input file
    #[argh(positional)]
    input: String,

    /// dump all metadata
    #[argh(switch, short = 'd')]
    dump: bool,

    /// IMU orientation (XYZ, ZXY etc, lowercase is negative, eg. xZy)
    #[argh(option)]
    imuo: Option<String>,
}

fn main() {
    let opts: Opts = argh::from_env();
    let _time = Instant::now();

    let mut stream = std::fs::File::open(&opts.input).unwrap();
    let filesize = stream.metadata().unwrap().len() as usize;

    let input = Input::from_stream(&mut stream, filesize, &opts.input, |_|(), Arc::new(AtomicBool::new(false))).unwrap();

    println!("Detected camera: {} {}", input.camera_type(), input.camera_model().unwrap_or(&"".into()));

    let samples = input.samples.as_ref().unwrap();

    if opts.dump {
        for info in samples {
            if info.tag_map.is_none() { continue; }
            let grouped_tag_map = info.tag_map.as_ref().unwrap();

            for (group, map) in grouped_tag_map {
                for (tagid, taginfo) in map {
                    println!("{: <25} {: <25} {: <50}: {}", format!("{}", group), format!("{}", tagid), taginfo.description, &taginfo.value.to_string());
                }
            }
        }
    }

    let imu_data = util::normalized_imu(&input, opts.imuo).unwrap();

    let mut csv = String::with_capacity(2*1024*1024);
    csv.push_str(r#"Clock:Tick, Dji data decorder by wuyi.zhang"#);
    csv.push('\n');

    // crate::try_block!({
    //     let map = samples.get(0)?.tag_map.as_ref()?;
    //     let json = (map.get(&GroupId::Default)?.get_t(TagId::Metadata) as Option<&serde_json::Value>)?;
    //     for (k, v) in json.as_object()? {
    //         csv.push('"');
    //         csv.push_str(&k.to_string());
    //         csv.push_str("\",");
    //         csv.push_str(&v.to_string());
    //         csv.push('\n');
    //     }
    // });

    // imu dir : yxZ
    csv.push_str(r#"Clock:offsetTime,IMU_ATTI(0):gyroX,IMU_ATTI(0):gyroY,IMU_ATTI(0):gyroZ,IMU_ATTI(0):accelX,IMU_ATTI(0):accelY,IMU_ATTI(0):accelZ"#);
    csv.push('\n');
    for v in imu_data {
        if v.gyro.is_some() || v.accl.is_some() {
            let gyro = v.gyro.unwrap_or_default();
            let accl = v.accl.unwrap_or_default();  
            // 前x 右y 下z
            csv.push_str(&format!("{},{},{},{},{},{},{}\n", (v.timestamp_ms / 1000000.0), gyro[0], gyro[1], gyro[2], accl[0] * 0.102, accl[1] * 0.102, accl[2] * 0.102
            ));
        }
    }
    std::fs::write(&format!("{}.csv", std::path::Path::new(&opts.input).to_path_buf().to_string_lossy()), csv).unwrap();

    println!("Done in {:.3} ms", _time.elapsed().as_micros() as f64 / 1000.0);
}
