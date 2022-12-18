const CRC8_ATM_POLY: u8 = 0x07;

pub fn checksum(data: &[u8]) -> u8 {
    let mut sum: usize = 0x9B;
    for b in data {
        sum += *b as usize;
    }
    sum as u8
}

pub fn crc_8_atm(data: &[u8]) -> u8 {
    let mut crc: u8 = 0x00;

    for b in data {
        let mut byte = *b;

        for _n in 0..8 {
            if (crc >> 7) ^ (byte & 0x01) != 0 {
                crc = (crc << 1) ^ CRC8_ATM_POLY;
            } else {
                crc = crc << 1;
            }
            byte = byte >> 1;
        }
    }

    crc
}

#[cfg(test)]
mod tests {
    use super::{checksum, crc_8_atm};

    #[test]
    fn test_crc() {
        let c = crc_8_atm(&[0x05, 0x00, 0x00]);
        assert_eq!(c, 0x48, "{c:#04x}")
    }

    #[test]
    fn test_checksum() {
        let c = checksum(&[255, 213, 85, 236]);
        assert_eq!(c, 0xAF, "{c:#04x}")
    }
}
