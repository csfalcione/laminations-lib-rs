pub mod laminations {
    use num::rational::Ratio;
    use num::Integer;
    use std::cmp::{Eq, Ord, Ordering};
    use std::ops::Mul;

    pub trait GenInt: Integer + Mul + Copy {}
    impl<T: Integer + Mul + Copy> GenInt for T {}

    #[derive(Debug, PartialEq)]
    pub struct UnitFraction<T: Integer + Mul + Copy> {
        base: T,
        rational: Ratio<T>,
    }

    impl<T: GenInt> UnitFraction<T> {
        pub fn new(base: T, numerator: T, denominator: T) -> UnitFraction<T> {
            UnitFraction {
                base,
                rational: Ratio::new_raw(numerator % denominator, denominator),
            }
        }

        pub fn map_forward(&self) -> UnitFraction<T> {
            let denominator = *self.rational.denom();
            let numerator: T = *self.rational.numer() * self.base;
            UnitFraction::new(self.base, numerator, denominator)
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
            self.rational.cmp(&other.rational)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn simplifies() {
            assert_eq! {
                UnitFraction::<usize>::new(2,1,2),
                UnitFraction::<usize>::new(2,2,4),
            }
        }

        #[test]
        fn wraps() {
            assert_eq! {
                UnitFraction::<usize>::new(2,1,2),
                UnitFraction::<usize>::new(2,3,2),
            }
        }

        #[test]
        fn map_forward() {
            assert_eq!(
                UnitFraction::<usize>::new(2, 1, 4).map_forward(),
                UnitFraction::<usize>::new(2, 1, 2)
            );
        }
    }
}
