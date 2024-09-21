use anyhow::{anyhow, Result};

pub struct TakeExactly<I: Iterator> {
    iter: I,
    expected: usize,
    count: usize,
}

impl<I: Iterator> TakeExactly<I> {
    pub fn new(iter: I, expected: usize) -> Self {
        TakeExactly {
            iter,
            expected,
            count: 0,
        }
    }
}

impl<I, V> Iterator for TakeExactly<I>
where
    I: Iterator<Item = Result<V>>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let next_value = self.iter.next();

        if let Some(next_value) = next_value {
            self.count += 1;

            if self.count > self.expected {
                return Some(Err(anyhow!(
                    "Expected {} items but got {}",
                    self.expected,
                    self.count
                )));
            }

            Some(next_value)
        } else {
            if self.count < self.expected {
                return Some(Err(anyhow!(
                    "Expected {} items but got {}",
                    self.expected,
                    self.count
                )));
            }

            None
        }
    }
}

pub struct ZipLongest<I: Iterator, J: Iterator> {
    iter1: I,
    iter2: J,
}

impl<I: Iterator, J: Iterator> ZipLongest<I, J> {
    pub fn new(iter1: I, iter2: J) -> Self {
        Self { iter1, iter2 }
    }
}

impl<I: Iterator, J: Iterator> Iterator for ZipLongest<I, J> {
    type Item = (Option<I::Item>, Option<J::Item>);

    fn next(&mut self) -> Option<Self::Item> {
        let a = self.iter1.next();
        let b = self.iter2.next();
        if a.is_none() && b.is_none() {
            None
        } else {
            Some((a, b))
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::ZipLongest;

    use super::TakeExactly;

    #[test]
    fn test_take_exactly() -> Result<()> {
        {
            let iter = TakeExactly::new([Ok(1), Ok(2)].into_iter(), 2);
            assert_eq!(iter.count(), 2);
        }

        {
            let iter = TakeExactly::new([Ok(1), Ok(2)].into_iter(), 2);
            let result = iter.collect::<Result<Vec<_>>>();
            assert!(result.is_ok());
        }

        {
            let iter = TakeExactly::new([Ok(1), Ok(2), Ok(3)].into_iter(), 2);
            let result = iter.collect::<Result<Vec<_>>>();
            assert!(result.is_err());
        }

        {
            let iter = TakeExactly::new([Ok(1), Ok(2)].into_iter(), 3);
            let result = iter.collect::<Result<Vec<_>>>();
            assert!(result.is_err());
        }

        Ok(())
    }

    #[test]
    fn test_zip_longest() -> Result<()> {
        {
            let iter = ZipLongest::new([1, 2].into_iter(), [3, 4].into_iter());
            assert_eq!(iter.count(), 2);
        }

        {
            let iter = ZipLongest::new([1, 2, 3].into_iter(), [3, 4].into_iter());
            assert_eq!(iter.count(), 3);
        }

        {
            let iter = ZipLongest::new([1, 2].into_iter(), [3, 4, 5].into_iter());
            assert_eq!(iter.count(), 3);
        }

        Ok(())
    }
}
