pub struct DoubleRange {
    current: i32,
    end: i32,
    step: i32,
}

impl DoubleRange {
    pub fn new(start: i32, end: i32, step: i32) -> Self {
        Self {
            current: start,
            end,
            step: step * (end - start).signum(),
        }
    }
}

impl Iterator for DoubleRange {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = None;

        if self.current != self.end {
            next = Some(self.current);
            self.current += self.step;
        }

        next
    }
}

pub struct DoubleRangeInclusive {
    current: i32,
    end: i32,
    step: i32,
    consumed: bool,
}

impl DoubleRangeInclusive {
    pub fn new(start: i32, end: i32, step: i32) -> Self {
        Self {
            current: start,
            end,
            step: step * (end - start).signum(),
            consumed: false,
        }
    }
}

impl Iterator for DoubleRangeInclusive {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.consumed {
            return None;
        }

        if self.current == self.end {
            self.consumed = true;
        }

        let next = self.current;
        self.current += self.step;

        Some(next)
    }
}

#[cfg(test)]
mod test {
    use crate::utils::double_range::DoubleRangeInclusive;

    use super::DoubleRange;

    #[test]
    fn double_range() {
        let range = DoubleRange::new(0, 4, 1);
        assert_eq!(range.collect::<Vec<i32>>(), vec![0, 1, 2, 3]);

        let range = DoubleRange::new(4, 0, 1);
        assert_eq!(range.collect::<Vec<i32>>(), vec![4, 3, 2, 1]);

        let range = DoubleRange::new(0, 4, 2);
        assert_eq!(range.collect::<Vec<i32>>(), vec![0, 2]);

        let range = DoubleRange::new(4, 0, 2);
        assert_eq!(range.collect::<Vec<i32>>(), vec![4, 2]);
    }

    #[test]
    fn double_range_inclusive() {
        let range = DoubleRangeInclusive::new(0, 4, 1);
        assert_eq!(range.collect::<Vec<i32>>(), vec![0, 1, 2, 3, 4]);

        let range = DoubleRangeInclusive::new(4, 0, 1);
        assert_eq!(range.collect::<Vec<i32>>(), vec![4, 3, 2, 1, 0]);

        let range = DoubleRangeInclusive::new(0, 4, 2);
        assert_eq!(range.collect::<Vec<i32>>(), vec![0, 2, 4]);

        let range = DoubleRangeInclusive::new(4, 0, 2);
        assert_eq!(range.collect::<Vec<i32>>(), vec![4, 2, 0]);
    }
}
