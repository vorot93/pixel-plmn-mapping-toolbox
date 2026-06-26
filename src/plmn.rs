use crate::error::Error;
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plmn(u32);

impl Plmn {
    /// Wrap a raw encoded value, validating it fits in 24 bits.
    pub fn from_encoded(value: i32) -> Result<Self, Error> {
        if !(0..=0xFF_FFFF).contains(&value) {
            return Err(Error::PlmnOutOfRange(value));
        }
        Ok(Plmn(value as u32))
    }

    /// The raw encoded value (always a positive i32).
    pub fn to_encoded(self) -> i32 {
        self.0 as i32
    }
}

impl fmt::Display for Plmn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let h = format!("{:06x}", self.0);
        let c = h.as_bytes();
        // wire nibble layout: c = [M2 M1 N3 M3 N2 N1]
        // MCC = M1 M2 M3 ; MNC = N1 N2 [N3 unless it is the filler nibble F]
        write!(
            f,
            "{}{}{}-{}{}",
            c[1] as char, c[0] as char, c[3] as char, c[5] as char, c[4] as char
        )?;
        if c[2] != b'f' {
            write!(f, "{}", c[2] as char)?;
        }
        Ok(())
    }
}

impl FromStr for Plmn {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bad = || Error::PlmnFormat(s.to_string());
        let (mcc, mnc) = s.split_once('-').ok_or_else(bad)?;
        let mcc = mcc.as_bytes();
        let mnc = mnc.as_bytes();
        if mcc.len() != 3 || (mnc.len() != 2 && mnc.len() != 3) {
            return Err(bad());
        }
        if !mcc.iter().chain(mnc.iter()).all(|b| b.is_ascii_hexdigit()) {
            return Err(bad());
        }
        // A 3-digit MNC may not end in the filler nibble F (use the 2-digit form).
        if mnc.len() == 3 && mnc[2].eq_ignore_ascii_case(&b'f') {
            return Err(bad());
        }
        let lc = |b: u8| (b as char).to_ascii_lowercase();
        let n3 = if mnc.len() == 3 { lc(mnc[2]) } else { 'f' };
        // reassemble the wire layout: M2 M1 N3 M3 N2 N1
        let h: String = [
            lc(mcc[1]),
            lc(mcc[0]),
            n3,
            lc(mcc[2]),
            lc(mnc[1]),
            lc(mnc[0]),
        ]
        .into_iter()
        .collect();
        let value = u32::from_str_radix(&h, 16).map_err(|_| bad())?;
        Ok(Plmn(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VECTORS: &[(i32, &str)] = &[
        (197154, "302-220"),  // TELUS, 3-digit MNC
        (10090905, "999-99"), // placeholder, 2-digit MNC
        (5435408, "250-01"),  // RU, 2-digit MNC
        (2291967, "228-ff"),  // wildcard MNC
    ];

    #[test]
    fn known_vectors_round_trip() {
        for &(val, s) in VECTORS {
            assert_eq!(
                Plmn::from_encoded(val).unwrap().to_string(),
                s,
                "decode {val}"
            );
            assert_eq!(s.parse::<Plmn>().unwrap().to_encoded(), val, "encode {s}");
        }
    }

    #[test]
    fn parse_is_case_insensitive() {
        assert_eq!(
            "228-FF".parse::<Plmn>().unwrap(),
            "228-ff".parse::<Plmn>().unwrap()
        );
    }

    #[test]
    fn rejects_out_of_range() {
        assert!(matches!(
            Plmn::from_encoded(-1),
            Err(Error::PlmnOutOfRange(-1))
        ));
        assert!(matches!(
            Plmn::from_encoded(0x100_0000),
            Err(Error::PlmnOutOfRange(_))
        ));
    }

    #[test]
    fn rejects_bad_strings() {
        for s in [
            "302", "30-220", "3022-20", "302-2", "302-2222", "30g-220", "302-2g0", "302-22f",
        ] {
            assert!(s.parse::<Plmn>().is_err(), "should reject {s}");
        }
    }

    #[test]
    fn bijection_full_sweep() {
        for v in 0u32..=0xFF_FFFF {
            let p = Plmn::from_encoded(v as i32).unwrap();
            assert_eq!(p.to_string().parse::<Plmn>().unwrap(), p, "{v:#08x}");
        }
    }
}
