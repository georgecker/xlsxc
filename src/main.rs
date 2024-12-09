mod image_conversion;

use core::{panic, str};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use clap::Parser;
use image_conversion::ImageDataReaderManager;
use simple_xlsx_writer::{row, Row, WorkBook};

// Excel has a char limit of 32.000 per cell
// -> Split frame into own cells
// TODO - CLI piping
// TODO - Write Color in HexCode formate into cells

#[derive(Parser, Debug)]
struct CLIArgs {
    /// URL of the srouce youtube video
    #[arg(short, long)]
    src_url: String,

    /// Optional -> System temp dir can be overriden
    #[arg(short, long, env)]
    tempdir: String,

    /// Ratio of frame compression. A ratio of 10/1 means only every tenth frame will be extracted
    #[arg(short, long, default_value_t = String::from("20/1"))]
    ratio: String,

    /// Width (in cells) of the video in the excel file
    #[arg(long, default_value_t = 32)]
    width: u32,

    /// Height (in cells) of the video in the excel file
    #[arg(long, default_value_t = 32)]
    height: u32,

    /// Output path of the excel file
    #[arg(short, long)]
    out_path: String,
}

fn main() {
    let args = CLIArgs::parse();
    println!("{:?}", args);
    return;

    // CONFIG
    let src = "https://www.youtube.com/shorts/4ruabxT8nI4";
    let buff_path = "/Users/georgecker/Projects/xlsxc/data/";
    let extract_ratio = "15/1";
    let width = 50;
    let height = 50;

    let excel_path = "/Users/georgecker/test.xlsx";

    // Donwlaod video
    let path_output = buff_path.to_owned() + "temp";
    let frames_dir_path = buff_path.to_string() + "frames/";

    downlaod(src, &path_output);
    extract_frames(&frames_dir_path, &path_output, extract_ratio);

    let frames_dir_reader = fs::read_dir(&frames_dir_path)
        .unwrap_or_else(|_| panic!("File not found \"{}\"", &frames_dir_path));
    let mut ids = Vec::new();
    frames_dir_reader.for_each(|dir| {
        let path = dir.unwrap().path().to_str().unwrap().to_string();
        let name = path.split_terminator("/").last().unwrap().to_string();
        if name.starts_with('.') {
            return;
        }
        let (id_str, _) = name.split_once('.').unwrap();
        let id: usize = id_str.parse().unwrap_or_else(|_| {
            panic!(
                "Error while parsing \"{}\" to usize. (Path: {})",
                id_str, path
            )
        });
        ids.push(id);
    });
    ids.sort();

    let map: Arc<Mutex<HashMap<usize, Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut img_data_reader = ImageDataReaderManager::new(ids.clone(), 25);
    while img_data_reader
        .work(map.clone(), &frames_dir_path, width, height, false)
        .is_none()
    {}

    let id_count = ids.len();
    write_excel(map.clone(), id_count, width as usize, excel_path);
}

fn write_excel(
    map: Arc<Mutex<HashMap<usize, Vec<u8>>>>,
    id_count: usize,
    width: usize,
    excel_path: &str,
) {
    let mut file = File::create(excel_path).unwrap();
    let mut workbook = WorkBook::new(&file).unwrap();
    let sheet = workbook.get_new_sheet();
    sheet
        .write_sheet(|sheet_writer| {
            let map = map.lock().unwrap();
            // replace with id_count
            for i in 1..=id_count {
                let val = map.get(&i).unwrap();
                // let val_formatted = format!("{:?}", val);
                // let val_str = val_formatted.trim_matches(['[', ']']);

                let mut id_frame = String::new();
                id_frame.push_str(&format!("{}:", width));
                for chunk in val.chunks(3) {
                    id_frame.push_str(&format!("{:02X}{:02X}{:02X}", chunk[0], chunk[1], chunk[2]));
                }

                // let id_frame = format!("{}:{}", width, val_str);
                sheet_writer.write_row(row![(id_frame)]).unwrap();
            }
            Ok(())
        })
        .unwrap();

    workbook.finish().unwrap();
    file.flush().unwrap();
}

fn downlaod(src: &str, out_path: &str) {
    let mp4_format = "vcodec:h264,res:480,acodec:m4a";
    let full_out_path = format!("{}.mp4", out_path);
    if fs::exists(&full_out_path).is_ok_and(|e| e) {
        println!("Delete old video at \"{}\"", &full_out_path);
        fs::remove_file(&full_out_path).unwrap();
        wait_for_file_deletion(&full_out_path, Duration::from_millis(100)).unwrap();
    }

    println!("Downloading video from \"{}\"", src);
    let mut child = Command::new("yt-dlp")
        .arg(src)
        .arg("-S")
        .arg(mp4_format)
        .arg("-o")
        .arg(out_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    print_chiled_stdout(&mut child);
    child.wait().unwrap();
}

fn extract_frames(frames_dir_path: &str, out_path: &str, ratio: &str) {
    if fs::exists(frames_dir_path).is_ok_and(|e| e) {
        println!("Delete existing frame dir at \"{}\"", frames_dir_path);
        fs::remove_dir_all(frames_dir_path).unwrap();
        wait_for_file_deletion(frames_dir_path, Duration::from_millis(100)).unwrap();
    }

    println!("Create new frame dir at \"{}\"", frames_dir_path);
    fs::create_dir(frames_dir_path).unwrap();
    wait_for_file_creation(frames_dir_path, Duration::from_millis(100)).unwrap();

    let target = frames_dir_path.to_string() + "%d.bmp";
    let src_file = out_path.to_string() + ".mp4";
    println!("Extract frames from \"{}\" into \"{}\"", out_path, target);
    let mut child = Command::new("ffmpeg")
        .arg("-i")
        .arg(&src_file)
        .arg("-r")
        .arg(ratio)
        .arg(&target)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    print_chiled_stdout(&mut child);
    child.wait().unwrap();
}

fn print_chiled_stdout(child: &mut Child) {
    let stdout = child.stdout.as_mut().unwrap();
    let stdout_reader = BufReader::new(stdout);
    let stdout_lines = stdout_reader.lines();

    for line in stdout_lines {
        println!("{}", line.unwrap());
    }
}

fn wait_for_file_deletion(p: &str, d: Duration) -> Result<(), io::Error> {
    while fs::exists(p)? {
        println!(
            "File at \"{}\" still exists. Wait for termination (next poll in: {}ms)",
            p,
            d.as_millis()
        );
        sleep(d);
    }

    Ok(())
}

fn wait_for_file_creation(p: &str, d: Duration) -> Result<(), io::Error> {
    while !fs::exists(p)? {
        println!(
            "File at \"{}\" not created yet. Wait for creation (next poll in: {}ms)",
            p,
            d.as_millis()
        );
        sleep(d);
    }

    Ok(())
}
