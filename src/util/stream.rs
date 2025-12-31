use bytes::{Buf, Bytes};
use tokio_util::codec::Decoder;

pub struct BoundedFrameDecoder {
    max_frames: usize,
    current_frames: usize,
    child: Box<dyn Decoder<Item = Bytes, Error = anyhow::Error> + Send>
}

#[derive(Debug)]
pub enum BoundedFrameReadError {
    MaximumFrames,
    Downstream(anyhow::Error),
    IO(std::io::Error)
}

impl From<std::io::Error> for BoundedFrameReadError {
    fn from(value: std::io::Error) -> Self {
        BoundedFrameReadError::IO(value)
    }
}

impl Decoder for BoundedFrameDecoder {
    type Item = Bytes;
    type Error = BoundedFrameReadError;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if self.current_frames >= self.max_frames {
            Err(BoundedFrameReadError::MaximumFrames)
        } else {
            match self.child.decode(src) {
                Ok(item) => {
                    match item {
                        Some(data) => {
                            self.current_frames += 1;
                            Ok(Some(data))
                        },
                        None => Ok(None),
                    }
                },
                Err(ds) => Err(BoundedFrameReadError::Downstream(ds))
            }
        }
    }
}

impl BoundedFrameDecoder {
    pub fn new(max_frames: usize, child: Box<dyn Decoder<Item = Bytes, Error = anyhow::Error> + Send>) -> Self {
        Self {
            max_frames,
            current_frames: 0,
            child
        }
    }
}

pub struct LengthPrefixedDataDecoder {}

impl LengthPrefixedDataDecoder {
    const SIZE_BYTE_LENGTH : usize = 4;

    pub fn new() -> Self {
        Self {}
    }
}

impl Decoder for LengthPrefixedDataDecoder {
    type Item = Bytes;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < LengthPrefixedDataDecoder::SIZE_BYTE_LENGTH {
            Ok(None)
        } else {
            let size = u32::from_be_bytes(
                <[u8; 4]>::try_from(&src[..LengthPrefixedDataDecoder::SIZE_BYTE_LENGTH]).unwrap()
            ) as usize;
            if (src.len().checked_sub(LengthPrefixedDataDecoder::SIZE_BYTE_LENGTH).unwrap_or(0)) < size {
                Ok(None)
            } else {
                src.advance(LengthPrefixedDataDecoder::SIZE_BYTE_LENGTH);
                let data = src.split_to(size);
                Ok(Some(data.freeze()))
            }
        }
    }
}