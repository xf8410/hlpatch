//! Resource budgets shared by the observer and its HTTP export paths.
//! Preview limits apply only to derived metadata; original bytes belong in raw storage.
use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};

pub const PAGE_BYTES: usize = 256 * 1024;
pub const PAGE_RECORDS: usize = 64;
pub const PREVIEW_BYTES: usize = 256;
pub const CAPTURE_BYTES: usize = 16 * 1024 * 1024;
pub const RANGE_BYTES: u64 = 1024 * 1024;
pub const STREAM_BUFFER_BYTES: usize = 64 * 1024;
pub const EXPORT_CONNECTIONS: usize = 2;
pub const HTTP_CONNECTIONS: usize = 8;

/// An allocation or connection admission owned until its work is finished/dropped.
pub struct Permit<'a> { counter: &'a AtomicUsize, size: usize }

impl Drop for Permit<'_> {
    fn drop(&mut self) { self.counter.fetch_sub(self.size, Ordering::AcqRel); }
}

/// Admission uses a single CAS, including overflow checks; callers never wait for capacity.
pub fn reserve(counter: &AtomicUsize, size: usize, limit: usize) -> Option<Permit<'_>> {
    counter.fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
        current.checked_add(size).filter(|next| *next <= limit)
    }).ok().map(|_| Permit { counter, size })
}

/// A bounded UTF-8 display prefix. The full original value must be kept separately.
pub fn text_prefix(value: &str, bytes: usize) -> &str {
    let mut end = value.len().min(bytes);
    while !value.is_char_boundary(end) { end -= 1; }
    &value[..end]
}

/// A small hex preview, without allocating a full-size hex copy of a payload.
pub fn hex_preview(bytes: &[u8]) -> String {
    const HEX: &[u8] = b"0123456789abcdef";
    let count = bytes.len().min(PREVIEW_BYTES);
    let mut out = String::with_capacity(count * 2);
    for &byte in &bytes[..count] {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 15) as usize] as char);
    }
    out
}

/// Copy exactly this range with fixed scratch space and propagate disconnect/read errors.
pub fn copy_range<R: Read, W: Write>(input: &mut R, output: &mut W, length: u64) -> io::Result<u64> {
    if length > RANGE_BYTES { return Err(io::Error::new(io::ErrorKind::InvalidInput, "range exceeds byte budget")); }
    let mut remaining = length;
    let mut buffer = [0u8; STREAM_BUFFER_BYTES];
    while remaining != 0 {
        let want = remaining.min(buffer.len() as u64) as usize;
        let count = input.read(&mut buffer[..want])?;
        if count == 0 { return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "raw file changed during export")); }
        output.write_all(&buffer[..count])?;
        remaining -= count as u64;
    }
    Ok(length)
}

/// Write headers and body separately; this must never allocate a second whole body.
pub fn write_json<W: Write>(output: &mut W, status: &str, body: &str) -> io::Result<()> {
    write_body(output, status, "application/json; charset=utf-8", None, body)
}

/// Common response writer, with CRLF and UTF-8 byte length verified independently of routing.
pub fn write_body<W: Write>(output: &mut W, status: &str, content_type: &str, filename: Option<&str>, body: &str) -> io::Result<()> {
    if [status, content_type].iter().any(|v| v.contains(['\r','\n']))
        || filename.is_some_and(|v| v.contains(['\r','\n','"','\\'])) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,"invalid response header"));
    }
    write!(output, "HTTP/1.1 {}\r\nContent-Type: {}\r\n", status, content_type)?;
    if let Some(name) = filename { write!(output,"Content-Disposition: attachment; filename=\"{}\"\r\n",name)?; }
    write!(output, "Content-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n", body.len())?;
    output.write_all(body.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_budget_releases_on_error_and_drop() {
        let budget = AtomicUsize::new(0);
        let first = reserve(&budget, 8, 16).unwrap();
        let second = reserve(&budget, 8, 16).unwrap();
        assert!(reserve(&budget, 1, 16).is_none());
        assert!(reserve(&budget, usize::MAX, 16).is_none());
        drop(first);
        assert_eq!(budget.load(Ordering::Acquire), 8);
        drop(second);
        assert_eq!(budget.load(Ordering::Acquire), 0);
    }

    #[test]
    fn concurrent_admission_never_exceeds_limit() {
        let budget = AtomicUsize::new(0);
        std::thread::scope(|scope| {
            for _ in 0..8 {
                let counter = &budget;
                scope.spawn(move || {
                    for _ in 0..1000 {
                        if let Some(_permit) = reserve(counter, 1, 2) {
                            assert!(counter.load(Ordering::Acquire) <= 2);
                            std::thread::yield_now();
                        }
                    }
                });
            }
        });
        assert_eq!(budget.load(Ordering::Acquire), 0);
    }

    struct PatternReader { remaining: usize, position: usize, largest_read: usize }
    impl Read for PatternReader {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            self.largest_read = self.largest_read.max(out.len());
            let count = out.len().min(self.remaining);
            for (offset, byte) in out[..count].iter_mut().enumerate() { *byte = ((self.position + offset) % 251) as u8; }
            self.position += count; self.remaining -= count;
            Ok(count)
        }
    }
    struct CheckingWriter { position: usize, disconnect_at: Option<usize> }
    impl Write for CheckingWriter {
        fn write(&mut self, input: &[u8]) -> io::Result<usize> {
            if self.disconnect_at.is_some_and(|limit| self.position >= limit) {
                return Err(io::ErrorKind::BrokenPipe.into());
            }
            for (offset, &byte) in input.iter().enumerate() { assert_eq!(byte, ((self.position + offset) % 251) as u8); }
            self.position += input.len(); Ok(input.len())
        }
        fn flush(&mut self) -> io::Result<()> { Ok(()) }
    }

    #[test]
    fn raw_files_1_16_64_mib_remain_exact_with_64_kib_scratch() {
        for mib in [1, 16, 64] {
            let size = mib * 1024 * 1024;
            let mut input = PatternReader { remaining: size, position: 0, largest_read: 0 };
            let mut output = CheckingWriter { position: 0, disconnect_at: None };
            while input.remaining != 0 {
                let length = input.remaining.min(RANGE_BYTES as usize) as u64;
                assert_eq!(copy_range(&mut input, &mut output, length).unwrap(), length);
            }
            assert_eq!(output.position, size);
            assert_eq!(input.largest_read, STREAM_BUFFER_BYTES);
        }
    }

    #[test]
    fn disconnect_and_short_file_are_errors_not_success() {
        let mut input = PatternReader { remaining: RANGE_BYTES as usize, position: 0, largest_read: 0 };
        let mut output = CheckingWriter { position: 0, disconnect_at: Some(STREAM_BUFFER_BYTES) };
        assert_eq!(copy_range(&mut input, &mut output, RANGE_BYTES).unwrap_err().kind(), io::ErrorKind::BrokenPipe);
        assert_eq!(copy_range(&mut &b"x"[..], &mut io::sink(), 2).unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
        assert_eq!(copy_range(&mut io::empty(), &mut io::sink(), RANGE_BYTES + 1).unwrap_err().kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn preview_does_not_copy_or_alter_full_payload() {
        let raw = vec![0xab; 2 * 1024 * 1024];
        assert_eq!(hex_preview(&raw), "ab".repeat(PREVIEW_BYTES));
        assert_eq!(raw.len(), 2 * 1024 * 1024);
        assert_eq!(text_prefix("中文text", 4), "中");
    }

    #[test]
    fn response_headers_use_crlf_and_utf8_byte_lengths() {
        let mut wire=Vec::new();
        let body="{\"text\":\"中文\"}";
        write_body(&mut wire,"200 OK","application/json",Some("capture.json"),body).unwrap();
        let wire=String::from_utf8(wire).unwrap();
        let (headers,payload)=wire.split_once("\r\n\r\n").unwrap();
        assert_eq!(payload,body);
        assert!(headers.contains(&format!("Content-Length: {}",body.len())));
        assert!(headers.contains("filename=\"capture.json\""));
        assert!(!headers.replace("\r\n", "").contains('\n'));
        assert!(write_body(&mut Vec::new(),"200 OK","application/json",Some("x\r\nInjected: y"),body).is_err());
    }
}
