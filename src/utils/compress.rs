use std::{io, io::Write};

use flate2::{
    write::{DeflateDecoder, DeflateEncoder},
    Compression,
};

pub fn encode(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    encoder.finish()
}

pub fn decode(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(Vec::new());
    decoder.write_all(data)?;
    decoder.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let data: &[u8] = "Hello world, I am aiter.".as_bytes();

        let encoded_data = encode(data).unwrap();
        assert_eq!(
            encoded_data,
            vec![
                243, 72, 205, 201, 201, 87, 40, 207, 47, 202, 73, 209, 81, 240, 84, 72, 204, 85,
                72, 204, 44, 73, 45, 210, 3, 0
            ]
        );

        let decoded_data = decode(&encoded_data).unwrap();
        assert_eq!(data, &decoded_data);
    }
}
