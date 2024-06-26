use super::*;

pub fn encode_to_vec(mut n: u128, v: &mut Vec<u8>) {
  while n >> 7 > 0 {
    v.push(n.to_le_bytes()[0] | 0b1000_0000);
    n >>= 7;
  }
  v.push(n.to_le_bytes()[0]);
}

pub fn decode(buffer: &[u8]) -> Result<(u128, usize), Error> {
  let mut n = 0u128;
  let mut i = 0;

  loop {
    if i < buffer.len() {
      let b = buffer[i] as u128;
      n *= 128;

      if b < 128 {
        return Ok((n + b, i + 1));
      }

      n = n + b - 127;

      i += 1;
    } else {
      return Ok((n, i));
    }
  }
}

pub fn encode(n: u128) -> Vec<u8> {
  let mut v = Vec::new();
  encode_to_vec(n, &mut v);
  v
}

#[derive(PartialEq, Debug)]
pub enum Error {
  Overlong,
  Overflow,
  Unterminated,
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Overlong => write!(f, "too long"),
      Self::Overflow => write!(f, "overflow"),
      Self::Unterminated => write!(f, "unterminated"),
    }
  }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn zero_round_trips_successfully() {
    let n = 0;
    let encoded = encode(n);
    let (decoded, length) = decode(&encoded).unwrap();
    assert_eq!(decoded, n);
    assert_eq!(length, encoded.len());
  }

  #[test]
  fn u128_max_round_trips_successfully() {
    let n = u128::MAX;
    let encoded = encode(n);
    let (decoded, length) = decode(&encoded).unwrap();
    assert_eq!(decoded, n);
    assert_eq!(length, encoded.len());
  }

  #[test]
  fn powers_of_two_round_trip_successfully() {
    for i in 0..128 {
      let n = 1 << i;
      let encoded = encode(n);
      let (decoded, length) = decode(&encoded).unwrap();
      assert_eq!(decoded, n);
      assert_eq!(length, encoded.len());
    }
  }

  #[test]
  fn alternating_bit_strings_round_trip_successfully() {
    let mut n = 0;

    for i in 0..129 {
      n = n << 1 | (i % 2);
      let encoded = encode(n);
      let (decoded, length) = decode(&encoded).unwrap();
      assert_eq!(decoded, n);
      assert_eq!(length, encoded.len());
    }
  }

  #[test]
  fn varints_may_not_be_longer_than_19_bytes() {
    const VALID: [u8; 19] = [
      128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 0,
    ];

    const INVALID: [u8; 20] = [
      128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
      128, 0,
    ];

    assert_eq!(decode(&VALID), Ok((0, 19)));
    assert_eq!(decode(&INVALID), Err(Error::Overlong));
  }

  #[test]
  fn varints_may_not_overflow_u128() {
    assert_eq!(
      decode(&[
        128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
        64,
      ]),
      Err(Error::Overflow)
    );
    assert_eq!(
      decode(&[
        128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
        32,
      ]),
      Err(Error::Overflow)
    );
    assert_eq!(
      decode(&[
        128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
        16,
      ]),
      Err(Error::Overflow)
    );
    assert_eq!(
      decode(&[
        128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
        8,
      ]),
      Err(Error::Overflow)
    );
    assert_eq!(
      decode(&[
        128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
        4,
      ]),
      Err(Error::Overflow)
    );
    assert_eq!(
      decode(&[
        128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
        2,
      ]),
      Ok((2u128.pow(127), 19))
    );
  }

  #[test]
  fn varints_must_be_terminated() {
    assert_eq!(decode(&[128]), Err(Error::Unterminated));
  }
}
