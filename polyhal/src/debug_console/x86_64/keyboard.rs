use x86::io::inb;

const SCAN_CODE_TO_ASCII: [u8; 58] = [
    0, 27, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'+', 0x7f, b'\t',
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']', 10, 0, b'a', b's',
    b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'', b'`', 0, b'\\', b'z', b'x', b'c', b'v',
    b'b', b'n', b'm', b',', b'.', b'/', 0, b'*', 0, b' ',
];

/// Get the key from the keyboard.
pub(super) fn get_key() -> Option<u8> {
    unsafe {
        match inb(0x64) & 1 != 0 {
            true => SCAN_CODE_TO_ASCII.get(inb(0x60) as usize).cloned(),
            false => None,
        }
    }
}
