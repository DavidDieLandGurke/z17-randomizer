mod de;
mod ser;

pub use de::{from_bytes, Deserializer};
pub use ser::{to_writer, Serializer};

crate::int_map! {
    Kind(u8) {
        String = 0xA0,
        Array = 0xC0,
        Map = 0xC1,
        Strings = 0xC2,
        Boolean = 0xD0,
        Integer = 0xD1,
        Float = 0xD2,
        Null = 0xFF,
    }
}

const MAGIC: &[u8; 2] = b"YB";
const VERSION: u16 = 1;

#[cfg(test)]
mod tests {
    pub mod data {
        pub const ARRAY: &[u8] = &[
            b'Y', b'B', 0x01, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x10, 0x0, 0x0, 0x0,
            0xC0, 0x01, 0, 0, 0xD1, 0, 0, 0, 0x67, 0x45, 0x23, 0x01,
        ];
        pub const NESTED_ARRAY: &[u8] = &[
            b'Y', b'B', 0x01, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x10, 0x0, 0x0, 0x0,
            0xC0, 0x01, 0, 0, 0xC0, 0, 0, 0, 0x1C, 0x0, 0x0, 0x0, 0xC0, 0x01, 0x0, 0x0, 0xD1, 0, 0,
            0, 0x67, 0x45, 0x23, 0x01,
        ];
        pub const MAP: &[u8] = &[
            b'Y', b'B', 0x1, 0x0, 0x10, 0x0, 0x0, 0x0, 0x28, 0x0, 0x0, 0x0, 0x40, 0x0, 0x0, 0x0,
            0xC2, 0x02, 0x0, 0x0, 0x10, 0x0, 0x0, 0x0, 0x14, 0x0, 0x0, 0x0, 0x18, 0x0, 0x0, 0x0,
            b'b', b'a', b'r', 0, b'f', b'o', b'o', 0, 0xC2, 0x02, 0x0, 0x0, 0x10, 0x0, 0x0, 0x0,
            0x14, 0x0, 0x0, 0x0, 0x18, 0x0, 0x0, 0x0, b'B', b'a', b'r', 0, b'F', b'o', b'o', 0,
            0xC1, 0x02, 0x0, 0x0, 0x01, 0x0, 0x0, 0xA0, 0x1, 0x0, 0x0, 0x0, 0x00, 0x0, 0x0, 0xA0,
            0x0, 0x0, 0x0, 0x0,
        ];
    }
}
