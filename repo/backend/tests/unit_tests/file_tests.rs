//! File validation + fingerprint unit tests.

use backend::services::file_service::FileService;

const PDF_BYTES: &[u8] = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\nrest of file";
const JPEG_BYTES: &[u8] = &[
    0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F', 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
];
const PNG_BYTES: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, b'I', b'H', b'D', b'R',
];
const EXE_BYTES: &[u8] = &[0x4D, 0x5A, 0x90, 0x00, 0x03, 0x00, 0x00, 0x00];

#[test]
fn test_pdf_magic_bytes_accepted() {
    FileService::validate_file(PDF_BYTES, "application/pdf").unwrap();
}

#[test]
fn test_jpeg_magic_bytes_accepted() {
    FileService::validate_file(JPEG_BYTES, "image/jpeg").unwrap();
}

#[test]
fn test_png_magic_bytes_accepted() {
    FileService::validate_file(PNG_BYTES, "image/png").unwrap();
}

#[test]
fn test_exe_magic_bytes_rejected() {
    let err = FileService::validate_file(EXE_BYTES, "application/pdf").unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("invalid")
            || msg.contains("MIME")
            || msg.to_lowercase().contains("unknown"),
        "got: {}",
        msg
    );
}

#[test]
fn test_file_with_pdf_extension_but_exe_magic_rejected() {
    // Even if the caller swears the upload is a PDF, magic-number inspection
    // wins and the EXE bytes are rejected. The detected MIME (or unknown)
    // never matches "application/pdf".
    let err = FileService::validate_file(EXE_BYTES, "application/pdf").unwrap_err();
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

#[test]
fn test_sha256_fingerprint_consistent() {
    let a = FileService::fingerprint(b"hello world");
    let b = FileService::fingerprint(b"hello world");
    assert_eq!(a, b, "fingerprint must be deterministic");
    assert_ne!(
        FileService::fingerprint(b"hello"),
        FileService::fingerprint(b"world"),
        "different inputs must hash differently"
    );
    assert_eq!(a.len(), 64, "SHA-256 hex is 64 chars");
}
