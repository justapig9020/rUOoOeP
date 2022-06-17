pub mod queue;

pub fn raw_to_u32_big_endian(vec: &[u8]) -> u32 {
    let mut ret = 0;
    for byte in vec {
        ret <<= 8;
        ret |= *byte as u32;
    }
    ret
}

// TODO: generalize this function
pub fn u32_to_raw_big_endian(mut val: u32) -> Vec<u8> {
    let mut ret = Vec::with_capacity(4);
    for _ in 0..4 {
        let byte = (val & 0xff) as u8;
        ret.push(byte);
        val >>= 8;
    }
    ret.reverse();
    ret
}

#[cfg(test)]
mod covert_functions {
    const U32: u32 = 0x0A0B0C0D;
    const RAW: [u8; 4] = [0xA, 0xB, 0xC, 0xD];
    use super::*;
    #[test]
    fn test_raw_data_to_u32() {
        let got = raw_to_u32_big_endian(&RAW);
        let expect = U32;
        assert_eq!(got, expect);
    }

    #[test]
    fn test_u32_to_raw() {
        let got = u32_to_raw_big_endian(U32);
        let expect = RAW;
        assert_eq!(&got, &expect);
    }
}
