// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

static BIT_MASK: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

static POPCOUNT_TABLE: [u8; 256] = [
    0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
    3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 4, 5, 5, 6, 5, 6, 6, 7, 5, 6, 6, 7, 6, 7, 7, 8,
];

/// Returns the nearest number that is `>=` than `num` and is a multiple of 64
#[inline]
pub fn round_upto_multiple_of_64(num: i64) -> i64 {
    round_upto_power_of_2(num, 64)
}

/// Returns the nearest multiple of `factor` that is `>=` than `num`. Here `factor` must
/// be a power of 2.
fn round_upto_power_of_2(num: i64, factor: i64) -> i64 {
    debug_assert!(factor > 0 && (factor & (factor - 1)) == 0);
    (num + (factor - 1)) & !(factor - 1)
}

/// Returns whether bit at position `i` in `data` is set or not
#[inline]
pub fn get_bit(data: &[u8], i: i64) -> bool {
    (data[(i >> 3) as usize] & BIT_MASK[(i & 7) as usize]) != 0
}

/// Returns whether bit at position `i` in `data` is set or not.
///
/// Note this doesn't do any bound checking, for performance reason. The caller is
/// responsible to guarantee that `i` is within bounds.
#[inline]
pub unsafe fn get_bit_raw(data: *const u8, i: i64) -> bool {
    (*data.offset((i >> 3) as isize) & BIT_MASK[(i & 7) as usize]) != 0
}

/// Sets bit at position `i` for `data`
#[inline]
pub fn set_bit(data: &mut [u8], i: i64) {
    data[(i >> 3) as usize] |= BIT_MASK[(i & 7) as usize]
}

/// Returns the number of 1-bits in `data`
#[inline]
pub fn count_set_bits(data: &[u8]) -> i64 {
    let mut count: i64 = 0;
    for u in data {
        count += POPCOUNT_TABLE[*u as usize] as i64;
    }
    count
}

/// Returns the number of 1-bits in `data`, starting from `offset`.
#[inline]
pub fn count_set_bits_offset(data: &[u8], offset: i64) -> i64 {
    debug_assert!(offset <= (data.len() * 8) as i64);

    let start_byte_pos = (offset >> 3) as usize;
    let start_bit_pos = offset & 7;

    if start_bit_pos == 0 {
        count_set_bits(&data[start_byte_pos..])
    } else {
        let mut result = 0;
        result += count_set_bits(&data[start_byte_pos + 1..]);
        for i in start_bit_pos..8 {
            if get_bit(&data[start_byte_pos..start_byte_pos + 1], i as i64) {
                result += 1;
            }
        }
        result
    }
}

/// Returns the ceil of `value`/`divisor`
#[inline]
pub fn ceil(value: i64, divisor: i64) -> i64 {
    let mut result = value / divisor;
    if value % divisor != 0 {
        result += 1
    };
    result
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng};
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_round_upto_multiple_of_64() {
        assert_eq!(0, round_upto_multiple_of_64(0));
        assert_eq!(64, round_upto_multiple_of_64(1));
        assert_eq!(64, round_upto_multiple_of_64(63));
        assert_eq!(64, round_upto_multiple_of_64(64));
        assert_eq!(128, round_upto_multiple_of_64(65));
        assert_eq!(192, round_upto_multiple_of_64(129));
    }

    #[test]
    fn test_get_bit() {
        // 00001101
        assert_eq!(true, get_bit(&[0b00001101], 0));
        assert_eq!(false, get_bit(&[0b00001101], 1));
        assert_eq!(true, get_bit(&[0b00001101], 2));
        assert_eq!(true, get_bit(&[0b00001101], 3));

        // 01001001 01010010
        assert_eq!(true, get_bit(&[0b01001001, 0b01010010], 0));
        assert_eq!(false, get_bit(&[0b01001001, 0b01010010], 1));
        assert_eq!(false, get_bit(&[0b01001001, 0b01010010], 2));
        assert_eq!(true, get_bit(&[0b01001001, 0b01010010], 3));
        assert_eq!(false, get_bit(&[0b01001001, 0b01010010], 4));
        assert_eq!(false, get_bit(&[0b01001001, 0b01010010], 5));
        assert_eq!(true, get_bit(&[0b01001001, 0b01010010], 6));
        assert_eq!(false, get_bit(&[0b01001001, 0b01010010], 7));
        assert_eq!(false, get_bit(&[0b01001001, 0b01010010], 8));
        assert_eq!(true, get_bit(&[0b01001001, 0b01010010], 9));
        assert_eq!(false, get_bit(&[0b01001001, 0b01010010], 10));
        assert_eq!(false, get_bit(&[0b01001001, 0b01010010], 11));
        assert_eq!(true, get_bit(&[0b01001001, 0b01010010], 12));
        assert_eq!(false, get_bit(&[0b01001001, 0b01010010], 13));
        assert_eq!(true, get_bit(&[0b01001001, 0b01010010], 14));
        assert_eq!(false, get_bit(&[0b01001001, 0b01010010], 15));
    }

    #[test]
    fn test_get_bit_raw() {
        const NUM_BYTE: usize = 10;
        let mut buf = vec![0; NUM_BYTE];
        let mut expected = vec![];
        let mut rng = thread_rng();
        for i in 0..8 * NUM_BYTE {
            let b = rng.gen_bool(0.5);
            expected.push(b);
            if b {
                set_bit(&mut buf[..], i as i64)
            }
        }

        let raw_ptr = buf.as_ptr();
        for (i, b) in expected.iter().enumerate() {
            unsafe {
                assert_eq!(*b, get_bit_raw(raw_ptr, i as i64));
            }
        }
    }

    #[test]
    fn test_set_bit() {
        let mut b = [0b00000000];
        set_bit(&mut b, 0);
        assert_eq!([0b00000001], b);
        set_bit(&mut b, 2);
        assert_eq!([0b00000101], b);
        set_bit(&mut b, 5);
        assert_eq!([0b00100101], b);
    }

    #[test]
    fn test_get_set_bit_roundtrip() {
        const NUM_BYTES: usize = 10;
        const NUM_SETS: usize = 10;

        let mut buffer: [u8; NUM_BYTES * 8] = [0; NUM_BYTES * 8];
        let mut v = HashSet::new();
        let mut rng = thread_rng();
        for _ in 0..NUM_SETS {
            let offset = rng.gen_range(0, 8 * NUM_BYTES);
            v.insert(offset);
            set_bit(&mut buffer[..], offset as i64);
        }
        for i in 0..NUM_BYTES * 8 {
            assert_eq!(v.contains(&i), get_bit(&buffer[..], i as i64));
        }
    }

    #[test]
    fn test_count_bits_slice() {
        assert_eq!(0, count_set_bits(&[0b00000000]));
        assert_eq!(8, count_set_bits(&[0b11111111]));
        assert_eq!(3, count_set_bits(&[0b00001101]));
        assert_eq!(6, count_set_bits(&[0b01001001, 0b01010010]));
    }

    #[test]
    fn test_count_bits_offset_slice() {
        assert_eq!(8, count_set_bits_offset(&[0b11111111], 0));
        assert_eq!(5, count_set_bits_offset(&[0b11111111], 3));
        assert_eq!(0, count_set_bits_offset(&[0b11111111], 8));
        assert_eq!(16, count_set_bits_offset(&[0b11111111, 0b11111111], 0));
        assert_eq!(13, count_set_bits_offset(&[0b11111111, 0b11111111], 3));
        assert_eq!(8, count_set_bits_offset(&[0b11111111, 0b11111111], 8));
        assert_eq!(5, count_set_bits_offset(&[0b11111111, 0b11111111], 11));
        assert_eq!(0, count_set_bits_offset(&[0b11111111, 0b11111111], 16));
    }

    #[test]
    fn test_ceil() {
        assert_eq!(ceil(0, 1), 0);
        assert_eq!(ceil(1, 1), 1);
        assert_eq!(ceil(1, 2), 1);
        assert_eq!(ceil(1, 8), 1);
        assert_eq!(ceil(7, 8), 1);
        assert_eq!(ceil(8, 8), 1);
        assert_eq!(ceil(9, 8), 2);
        assert_eq!(ceil(9, 9), 1);
        assert_eq!(ceil(10000000000, 10), 1000000000);
        assert_eq!(ceil(10, 10000000000), 1);
        assert_eq!(ceil(10000000000, 1000000000), 10);
    }
}
