use std::{
    collections::HashMap,
    io::{stdout, Write},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use image::{imageops, GenericImageView, ImageReader, Pixel};

// TODO - Loadingbar in console
pub struct ImageDataReaderManager {
    staged_ids: Vec<usize>,
    chunk_size: usize,
    handles: Vec<JoinHandle<()>>,
}

impl ImageDataReaderManager {
    pub fn new(staged_ids: Vec<usize>, chunk_size: usize) -> Self {
        Self {
            chunk_size,
            staged_ids,
            handles: Vec::new(),
        }
    }

    pub fn work(
        &mut self,
        map: Arc<Mutex<HashMap<usize, Vec<u8>>>>,
        frames_dir_path: &str,
        to_width: u32,
        to_hight: u32,
        is_debug: bool,
    ) -> Option<()> {
        while self.handles.len() < self.chunk_size {
            if self.staged_ids.is_empty() {
                break;
            }

            let std_out = stdout();
            let id = self.staged_ids.remove(0);
            let path = format!("{}{}.bmp", frames_dir_path, id);
            let map = map.clone();
            let handle = thread::spawn(move || {
                if is_debug {
                    std_out
                        .lock()
                        .write_fmt(format_args!("ID: {} -> Start extracting img\n", id))
                        .unwrap();
                }
                // 100 x 100 ist das limit wegen der max char pro cell von excel
                let data = ImageDataReaderManager::get_image_data(&path, to_width, to_hight);
                if is_debug {
                    std_out
                        .lock()
                        .write_fmt(format_args!(
                            "ID: {} -> Finished image processing\nAdd to map -> Aquireing lock\n",
                            id
                        ))
                        .unwrap();
                }
                let mut map = map.lock().unwrap();
                if is_debug {
                    std_out
                        .lock()
                        .write_fmt(format_args!("ID: {} -> Lock aquired!\n", id))
                        .unwrap();
                }
                map.insert(id, data);
                if is_debug {
                    std_out
                        .lock()
                        .write_fmt(format_args!("ID: {} -> Inserted into Map!\n", id))
                        .unwrap();
                }
            });

            self.handles.push(handle);
        }

        let mut index = 0;
        let mut current_count = self.handles.len();
        let mut std_out = stdout();
        while index < current_count {
            let handle = self.handles.remove(index);
            if handle.is_finished() {
                let res = handle.join();
                current_count -= 1;
                if is_debug {
                    std_out
                        .lock()
                        .write_fmt(format_args!(
                            "Handle count: {}\nResult: {:?}",
                            self.handles.len(),
                            res
                        ))
                        .unwrap();
                }
                std_out.lock().write_all(b"#").unwrap();
                std_out.flush().unwrap();
            } else {
                self.handles.insert(index, handle);
            }

            index += 1;
        }

        if self.handles.is_empty() {
            return Some(());
        }

        None
    }

    fn get_image_data(path: &str, to_width: u32, to_hight: u32) -> Vec<u8> {
        let img = ImageReader::open(path).unwrap().decode().unwrap();
        let img_resized = img.resize_exact(to_width, to_hight, imageops::FilterType::Nearest);
        let mut data = Vec::new();
        img_resized.pixels().for_each(|(_, _, color)| {
            color.to_rgb().0.iter().for_each(|b| data.push(*b));
        });

        data
    }
}
