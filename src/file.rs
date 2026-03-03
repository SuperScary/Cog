use std::fs;
use std::path::Path;

use chardetng::EncodingDetector;
use encoding_rs::{Encoding, UTF_8};

pub struct DecodedFile {
    pub text: String,
    pub encoding_name: String,
}

pub fn read_file_with_encoding_detection(path: &str) -> std::io::Result<DecodedFile> {
    let bytes = fs::read(Path::new(path))?;

    let mut detector = EncodingDetector::new();
    detector.feed(&bytes, true);

    let encoding = detector.guess(None, true);

    let (decoded, _, had_errors) = encoding.decode(&bytes);

    let text = if had_errors {
        let (fallback, _, _) = UTF_8.decode(&bytes);
        fallback.into_owned()
    } else {
        decoded.into_owned()
    };

    Ok(DecodedFile {
        text,
        encoding_name: encoding.name().to_string(),
    })
}

pub fn save_with_encoding(path: &str, text: &str, encoding_name: &str) -> std::io::Result<()> {
    let encoding = Encoding::for_label(encoding_name.as_bytes()).unwrap_or(encoding_rs::UTF_8);

    let (encoded, _, _) = encoding.encode(text);

    std::fs::write(path, &encoded)
}
