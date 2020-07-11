use internal::buffer;

// use bit_set::BitSet;

pub fn read_uint32(data: &mut buffer::Buffer) -> u32 {
    let bytes = data.read_multi(4);
    let mut result: u32 = 0;

    for i in (0..4).rev() {
        result += bytes[i] as u32;
        if i > 0 {
            result *= 256;
        }
    }
    result
}

pub fn read_uint64(data: &mut buffer::Buffer) -> u64 {
    let bytes = data.read_multi(8);
    let mut result: u64 = 0;

    for i in (0..8).rev() {
        result += bytes[i] as u64;
        if i > 0 {
            result *= 256;
        }
    }
    result
}

pub fn read_dyn_uint64(data: &mut buffer::Buffer) -> u64 {
    let first_byte: u64 = data.read() as u64;
    let mut mask: u64 = 0x80;
    let mut value: u64 = 0;
    for i in 0..8 {
        if (first_byte & mask) == 0 {
            return value | ((first_byte & (mask - 1)) << (8 * i));
        }
        let next_byte: u64 = data.read() as u64;
        value |= next_byte << (8 * i);
        mask >>= 1;
    }
    value
}

pub fn read_all_or_bits(data: &mut buffer::Buffer, size: u64) -> bit_set::BitSet {
    let all_defined = data.read() as u8;
    if all_defined != 0 {
        (0..(size as usize)).filter(|_| true).collect()
    } else {
        read_bits(data, size)
    }
}

pub fn read_bits(data: &mut buffer::Buffer, size: u64) -> bit_set::BitSet {
    println!("Read bits {}", size);
    let mut set = bit_set::BitSet::with_capacity(size as usize);
    let mut mask = 0;
    let mut cache = 0;
    for i in 0..size {
        if mask == 0 {
            mask = 0x80;
            cache = data.read();
        }
        if cache & mask != 0 {
            set.insert(i as usize);
        }
        mask >>= 1;
    }
    set
}

mod tests_uint32 {
    #[test]
    fn read_uint32_zero() {
        let mut buff = super::buffer::Buffer::new(&[0, 0, 0, 0]);
        let result = super::read_uint32(&mut buff);
        assert_eq!(result, 0);
    }

    #[test]
    fn read_uint32_257() {
        let mut buff = super::buffer::Buffer::new(&[1, 1, 0, 0]);
        let result = super::read_uint32(&mut buff);
        assert_eq!(result, 257);
    }

    #[test]
    fn read_uint32_4096() {
        let mut buff = super::buffer::Buffer::new(&[0, 16, 0, 0]);
        let result = super::read_uint32(&mut buff);
        assert_eq!(result, 4096);
    }

    #[test]
    fn read_two_numbers() {
        let mut buff = super::buffer::Buffer::new(&[0, 16, 0, 0, 0, 8, 0, 0]);
        let result1 = super::read_uint32(&mut buff);
        let result2 = super::read_uint32(&mut buff);
        assert_eq!(result1, 4096);
        assert_eq!(result2, 2048);
    }
}

mod tests_uint64 {
    #[test]
    fn read_uint64_zero() {
        let mut buff = super::buffer::Buffer::new(&[0, 0, 0, 0, 0, 0, 0, 0]);
        let result = super::read_uint64(&mut buff);
        assert_eq!(result, 0);
    }

    #[test]
    fn read_uint64_257() {
        let mut buff = super::buffer::Buffer::new(&[1, 2, 3, 4, 200, 201, 202, 203]);
        let result = super::read_uint64(&mut buff);
        assert_eq!(result, 14684771395892871681);
    }

    #[test]
    fn read_uint64_4096() {
        let mut buff = super::buffer::Buffer::new(&[0, 16, 0, 0, 0, 0, 0, 0]);
        let result = super::read_uint64(&mut buff);
        assert_eq!(result, 4096);
    }
}

mod tests_dyn_uint64 {
    #[test]
    fn read_real_uint64_zero() {
        let mut buff = super::buffer::Buffer::new(&[0]);
        let result = super::read_dyn_uint64(&mut buff);
        assert_eq!(result, 0);
    }

    #[test]
    fn read_real_uint64_2199140894112() {
        let mut buff = super::buffer::Buffer::new(&[250, 160, 5, 3, 7, 0, 0, 1, 3]);
        let result = super::read_dyn_uint64(&mut buff);
        assert_eq!(result, 2199140894112);
    }
}

mod tests_read_all_or_bits {
    #[test]
    fn all_bits_true() {
        let bitset = super::read_all_or_bits(&mut super::buffer::Buffer::new(&[1, 0]), 3);
        assert!(bitset.contains(0), "All bits should be true");
        assert!(bitset.contains(1), "All bits should be true");
        assert!(bitset.contains(2), "All bits should be true");
    }

    #[test]
    fn read_bits() {
        let bitset = super::read_all_or_bits(&mut super::buffer::Buffer::new(&[0, 128]), 4);
        let vec = bitset.into_bit_vec();
        assert_eq!(vec[0], true);
        assert_eq!(vec[1], false);
        assert_eq!(vec[2], false);
        assert_eq!(vec[3], false);
        assert_eq!(vec.len(), 4);
    }
}
