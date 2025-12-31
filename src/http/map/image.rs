use std::{io::Cursor, path::PathBuf, sync::Arc};

use bytes::Bytes;
use image::{GenericImageView, ImageReader};
use log::{error, warn};
use rgb::FromSlice;
use tokio::sync::{Semaphore, mpsc::{self, Sender}};

use crate::util::{r#macro::unwrap_helper, stream::{BoundedFrameDecoder, LengthPrefixedDataDecoder}};

pub struct ImageState {
    pub transmit: Arc<Sender<(String, Vec<u8>)>>
}

pub fn image_queue_processor(
    base_path: Arc<PathBuf>, transcode: bool, image_guard: Arc<Semaphore>
) -> mpsc::Sender<(String, Vec<u8>)> {
    let (tx, mut rx) = mpsc::channel::<(String, Vec<u8>)>(
        128
    );
    tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            submit_image(base_path.clone(), transcode, image_guard.clone(), data).await;
        }
    });
    tx
}

async fn submit_image(base_path: Arc<PathBuf>, transcode: bool, image_guard: Arc<Semaphore>, data: (String, Vec<u8>)) {
    let permit = image_guard.clone().acquire_owned().await.unwrap();
    tokio::task::spawn_blocking(move || {
        let _permit_use = permit;
        store_map_image(base_path, transcode, data);
    });
}

fn store_map_image(
    base_path: Arc<PathBuf>, 
    transcode: bool, 
    image_and_name: (String, Vec<u8>)
) {
    let (file_name, image) = image_and_name;
    if !transcode {
        let destination = base_path.join(format!("{}.png", file_name));
        std::fs::write(destination, image);
    } else {
        let destination = base_path.join(format!("{}.avif", file_name));
        let data = image;

        let mut image_reader = ImageReader::new(Cursor::new(data));
        image_reader.set_format(image::ImageFormat::Png);
        let image = match image_reader.decode() {
            Ok(image) => image,
            Err(e) => {
                error!("Could not save {} due to {:?}", file_name, e);
                return;
            },
        };
        let (width, height) = image.dimensions();
        let rgb_image = image.into_rgba8();
        let rgb_data = rgb_image.into_raw();
        let encoder = ravif::Encoder::new();
        let img = imgref::Img::new(rgb_data.as_rgba(), width as usize, height as usize);
        let avif_data = unwrap_helper::result_return_default!(encoder.encode_rgba(img), ());
        std::fs::write(destination, avif_data.avif_file);
    }
}

pub fn create_image_decoder() -> BoundedFrameDecoder {
    BoundedFrameDecoder::new(
        1024, 
        Box::new(
            LengthPrefixedDataDecoder::new()
        )
    )
}

pub fn parse_image_data(mut bytes: Bytes) -> (String, Vec<u8>)  {
    let image_name = read_length_prefixed_string(&mut bytes);
    (image_name, Vec::from(bytes))
}

fn read_length_prefixed_string(bytes: &mut Bytes) -> String {
    String::from_utf8(Vec::from(read_length_prefixed_data(bytes))).unwrap()
}

fn read_length_prefixed_data(bytes: &mut Bytes) -> Bytes {
    let arr = <[u8; 4]>::try_from(bytes.split_to(4).as_ref()).unwrap();
    let size = u32::from_be_bytes(arr);
    bytes.split_to(size as usize)
}