pub mod laminations {
    use num::pow::pow;
    use num::rational::Ratio;
    use std::cmp::{Eq, Ord, Ordering};
    use std::marker::PhantomData;

    pub type DefaultAlgebra = LaminationAlgebra<UnitFraction>;

    pub trait UnitNumber: Eq + Ord + Sized {
        fn parse_nary(base: u8, s: &str) -> Result<Self, String>;

        fn to_rational(&self, base: u8) -> Ratio<u128>;

        fn to_float(&self, base: u8) -> f64 {
            let rational = self.to_rational(base);
            let numerator: f64 = *rational.numer() as f64;
            let denominator: f64 = *rational.denom() as f64;
            numerator / denominator
        }
    }

    pub struct LaminationAlgebra<T: UnitNumber> {
        pub base: u8,
        _marker: PhantomData<T>,
    }

    impl<T: UnitNumber> LaminationAlgebra<T> {
        pub fn new(base: u8) -> Self {
            Self {
                base,
                _marker: PhantomData,
            }
        }

        pub fn parse(&self, s: &str) -> Result<T, String> {
            T::parse_nary(self.base, s)
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct UnitFraction {
        pub exact_num: u128,
        pub exact_len: u8,
        pub repeating_num: u128,
        pub repeating_len: u8,
    }
    
    impl UnitFraction {
        pub fn new(exact_num: u128, exact_len: u8, repeating_num: u128, repeating_len: u8) -> UnitFraction {
            UnitFraction {
                exact_num,
                exact_len,
                repeating_num,
                repeating_len,
            }
        }

    }

    impl UnitNumber for UnitFraction {
        fn parse_nary(base: u8, s: &str) -> Result<Self, String> {
            let (exact_digits, repeating_digits) = parse_digit_parts(base, s)?;

            let exact_num = value_from_digits(base, &exact_digits);
            let exact_len = exact_digits.len() as u8;
            let repeating_num = value_from_digits(base, &repeating_digits);
            let repeating_len = repeating_digits.len() as u8;

            Ok(UnitFraction::new(exact_num, exact_len, repeating_num, repeating_len))
        }

        fn to_rational(&self, base: u8) -> Ratio<u128> {
            let get_repeating_denominator = || -> u128 {
                let result = pow(base as u128, self.repeating_len as usize) - 1;
                if result == 0 {
                    return 1;
                }
                result
            };
    
            let repeating_denominator = get_repeating_denominator();
            let denominator = repeating_denominator * pow(base as u128, self.exact_len as usize);
            let numerator = repeating_denominator * self.exact_num + self.repeating_num;
    
            Ratio::new(numerator, denominator)
        }
    }

    impl Eq for UnitFraction {}

    impl Ord for UnitFraction {
        fn cmp(&self, other: &Self) -> Ordering {
            self.to_rational(2).cmp(&other.to_rational(2))
        }
    }

    impl PartialOrd for UnitFraction {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    pub fn parse_digit_parts(base: u8, s: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
        let parts: Vec<&str> = s.split('_').collect();

        if parts.len() > 2 {
            return Err(format!("`{}` contains more than one underscore", s));
        }

        let digit_splitter = if base < 10 { "" } else { "," };

        let parse_digits = |digits: &str| -> Result<Vec<u8>, String> {
            digits
                .split(digit_splitter)
                .filter(|digit| digit.len() > 0)
                .map(|digit: &str| {
                    digit
                        .parse::<u8>()
                        .map_err(|_| format!("{}: `{}` is not numerical", s, digit))
                })
                .collect()
        };

        Ok((
            parse_digits(parts[0])?,
            parse_digits(parts.get(1).unwrap_or(&""))?,
        ))
    }

    pub fn value_from_digits(base: u8, digits: &[u8]) -> u128 {
        digits
            .iter()
            .rev()
            .fold((0, 1), |(sum, exp), &digit| {
                (sum + (digit as u128) * exp, exp * (base as u128))
            })
            .0
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        type Fraction = UnitFraction;

        #[test]
        fn parse_ternary() {
            let ternary = DefaultAlgebra::new(3);

            assert_eq! {
                Fraction::new(0, 0, 0, 0),
                ternary.parse("_").unwrap(),
            }
            assert_eq! {
                Fraction::new(1, 1, 0, 0),
                ternary.parse("1_").unwrap(),
            }
            assert_eq! {
                Fraction::new(9, 3, 0, 0),
                ternary.parse("100").unwrap(),
            }
            assert_eq! {
                Fraction::new(9, 3, 0, 0),
                ternary.parse("100_").unwrap(),
            }
            assert_eq! {
                Fraction::new(0, 0, 9, 3),
                ternary.parse("_100").unwrap(),
            }
            assert_eq! {
                Fraction::new(1, 1, 9, 3),
                ternary.parse("1_100").unwrap(),
            }

            assert!(ternary.parse("1_100_").is_err());
            assert!(ternary.parse("1_o1").is_err())
        }

        #[test]
        fn parse_dozenal() {
            let dozenal = DefaultAlgebra::new(12);

            assert_eq! {
                Fraction::new(0, 0, 3, 1),
                dozenal.parse("_3").unwrap(),
            }
            assert_eq! {
                Fraction::new(1694, 3, 0, 0),
                dozenal.parse("11,9,2").unwrap(),
            }
            assert_eq! {
                Fraction::new(1694, 3, 0, 0),
                dozenal.parse("11,9,2_").unwrap(),
            }
            assert_eq! {
                Fraction::new(0, 0, 1694, 3),
                dozenal.parse("_11,9,2").unwrap(),
            }
            assert_eq! {
                Fraction::new(11, 1, 1694, 3),
                dozenal.parse("11_11,9,2").unwrap(),
            }
        }

        #[test]
        fn simplifies() {
            let ternary = DefaultAlgebra::new(3);
            let quaternary = DefaultAlgebra::new(4);

            let a1 = ternary.parse("_102").unwrap();
            let a2 = ternary.parse("1_021").unwrap();
            let a3 = ternary.parse("10_210").unwrap();
            let a4 = ternary.parse("102_102").unwrap();
            let a5 = ternary.parse("1021_021").unwrap();
            let b1 = ternary.parse("2_1").unwrap();
            assert_eq!(a1, a2);
            assert_eq!(a1, a3);
            assert_eq!(a1, a4);
            assert_eq!(a1, a5);
            assert_ne!(a1, b1);

            let c1 = ternary.parse("_1").unwrap();
            let c2 = ternary.parse("_111").unwrap();
            assert_eq!(c1, c2);

            let d1 = quaternary.parse("31_102").unwrap();
            let d2 = quaternary.parse("311021021_021").unwrap();
            assert_eq!(d1, d2);

            let f1 = ternary.parse("2_").unwrap();
            let f2 = ternary.parse("200_").unwrap();
            let f3 = ternary.parse("200_00").unwrap();
            assert_eq!(f1, f2);
            assert_eq!(f1, f3);

            let g1 = ternary.parse("_").unwrap();
            let g2 = ternary.parse("_2").unwrap();
            assert_eq!(g1, g2);
        }
    }
}
