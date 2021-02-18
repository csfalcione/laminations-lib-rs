pub mod laminations {
    use num::pow::pow;
    use num::rational::Ratio;
    use num::Integer;
    use std::cmp::{Eq, Ord, Ordering};
    use std::fmt::Display;
    use std::marker::PhantomData;
    use std::str::FromStr;

    pub trait GenInt: Integer + Copy + FromStr + From<u8> + Display {}
    impl<T: Integer + Copy + FromStr + From<u8> + Display> GenInt for T {}

    pub type DefaultAlgebra = LaminationAlgebra<u128, UnitFraction<u128>>;

    pub trait UnitNumber<T: GenInt>: Eq + Ord + Sized {
        fn parse_nary(base: T, s: &str) -> Result<Self, String>;
    }

    pub struct LaminationAlgebra<T: GenInt, U: UnitNumber<T>> {
        pub base: T,
        _marker: PhantomData<U>,
    }

    impl<T: GenInt, U: UnitNumber<T>> LaminationAlgebra<T, U> {
        pub fn new(base: T) -> Self {
            Self {
                base,
                _marker: PhantomData,
            }
        }

        pub fn parse(&self, s: &str) -> Result<U, String> {
            U::parse_nary(self.base, s)
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct UnitFraction<T: GenInt>(Ratio<T>);

    impl<T: GenInt> UnitFraction<T> {
        pub fn new(numerator: T, denominator: T) -> UnitFraction<T> {
            UnitFraction(Ratio::new(numerator % denominator, denominator))
        }
    }

    impl<T: GenInt> UnitNumber<T> for UnitFraction<T> {
        fn parse_nary(base: T, s: &str) -> Result<Self, String> {
            let zero = T::from(0);
            let one = T::from(1);
            let ten = T::from(10);

            let parts: Vec<&str> = s.split('_').collect();

            if parts.len() > 2 {
                return Err(format!("`{}` contains more than one underscore", s));
            }

            let digit_splitter = if base < ten { "" } else { "," };

            let parse_digits = |digits: &str| -> Result<Vec<T>, String> {
                digits
                    .split(digit_splitter)
                    .filter(|digit| digit.len() > 0)
                    .map(|digit: &str| {
                        digit
                            .parse::<T>()
                            .map_err(|_| format!("{}: `{}` is not numerical", s, digit))
                    })
                    .collect()
            };

            let exact_digits = parse_digits(parts[0])?;
            let repeating_digits = parse_digits(parts.get(1).unwrap_or(&""))?;

            let get_repeating_denominator = || -> T {
                let result = pow(base, repeating_digits.len()) - one;
                if result == zero {
                    return one;
                }
                result
            };

            let value_from_digits = |digits: Vec<T>| -> T {
                digits
                    .iter()
                    .rev()
                    .fold((zero, one), |(sum, exp), &digit| {
                        (sum + digit * exp, exp * base)
                    })
                    .0
            };

            let repeating_denominator = get_repeating_denominator();
            let denominator = repeating_denominator * pow(base, exact_digits.len());
            let numerator = repeating_denominator * value_from_digits(exact_digits)
                + value_from_digits(repeating_digits);

            Ok(Self::new(numerator, denominator))
        }
    }

    impl<T: GenInt> Eq for UnitFraction<T> {}

    impl<T: GenInt> PartialOrd for UnitFraction<T> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl<T: GenInt> Ord for UnitFraction<T> {
        fn cmp(&self, other: &Self) -> Ordering {
            self.0.cmp(&other.0)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        type Fraction = UnitFraction<u128>;

        #[test]
        fn parse_ternary() {
            let ternary = DefaultAlgebra::new(3);

            assert_eq! {
                Fraction::new(0, 1),
                ternary.parse("_").unwrap(),
            }
            assert_eq! {
                Fraction::new(1,3),
                ternary.parse("1_").unwrap(),
            }
            assert_eq! {
                Fraction::new(1,3),
                ternary.parse("100").unwrap(),
            }
            assert_eq! {
                Fraction::new(1,3),
                ternary.parse("100_").unwrap(),
            }
            assert_eq! {
                Fraction::new(9,26),
                ternary.parse("_100").unwrap(),
            }
            assert_eq! {
                Fraction::new(35,78),
                ternary.parse("1_100").unwrap(),
            }

            assert!(ternary.parse("1_100_").is_err());
            assert!(ternary.parse("1_o1").is_err())
        }

        #[test]
        fn parse_dozenal() {
            let dozenal = DefaultAlgebra::new(12);

            assert_eq! {
                Fraction::new(3, 11),
                dozenal.parse("_3").unwrap(),
            }
            assert_eq! {
                Fraction::new(847,864),
                dozenal.parse("11,9,2").unwrap(),
            }
            assert_eq! {
                Fraction::new(847,864),
                dozenal.parse("11,9,2_").unwrap(),
            }
            assert_eq! {
                Fraction::new(154,157),
                dozenal.parse("_11,9,2").unwrap(),
            }
            assert_eq! {
                Fraction::new(627,628),
                dozenal.parse("11_11,9,2").unwrap(),
            }
        }

        #[test]
        fn simplifies() {
            assert_eq! {
                Fraction::new(1,2),
                Fraction::new(2,4),
            }
        }

        #[test]
        fn wraps() {
            assert_eq! {
                Fraction::new(1,2),
                Fraction::new(3,2),
            }
        }
    }
}
