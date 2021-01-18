// Vec that's garunteed to be sorted for a binary search
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
pub struct SortedVec<T: Ord> {
    vec: Vec<T>,
}

impl<T: Ord> SortedVec<T> {
    pub fn new() -> Self {
        return Self { vec: Vec::new() };
    }

    #[inline]
    pub fn insert(&mut self, item: T) {
        match self.vec.binary_search(&item) {
            Ok(_) => (),
            Err(idx) => {
                self.vec.insert(idx, item);
            }
        }
    }

    pub fn position_of(&self, target: &T) -> Option<usize> {
        return match self.vec.binary_search(target) {
            Ok(idx) => Some(idx),
            Err(_) => None,
        };
    }

    // Simple binary search for O(log(n)) time complexity, quicker than hashmaps for small vecs
    #[inline]
    pub fn contains(&self, target: &T) -> bool {
        // return self.vec.binary_search(target).is_ok();
        let mut size = self.vec.len();
        if size == 0 {
            return false;
        }
        let mut base = 0_usize;

        while size > 1 {
            // mid: [base..size)
            let half = size / 2;
            let mid = base + half;
            unsafe {
                if self.vec.get_unchecked(mid) <= target {
                    base = mid;
                }
            }
            size -= half;
        }
        if unsafe { self.vec.get_unchecked(base) } == target {
            return true;
        } else {
            return false;
        }
    }

    #[inline]
    pub fn to_vec(self) -> Vec<T> {
        return self.vec;
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        return self.vec.iter();
    }

    // SAFETY: You have to remember to sort it after modifying values otherwise it won't work as planned and may cause UB in the binary search
    #[inline]
    pub unsafe fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        return self.vec.iter_mut();
    }

    #[inline]
    pub fn sort(&mut self) {
        self.vec.sort();
    }

    #[inline]
    pub fn sort_unstable(&mut self) {
        self.vec.sort_unstable();
    }
}

impl<T: Ord> From<Vec<T>> for SortedVec<T> {
    fn from(mut vec: Vec<T>) -> Self {
        vec.sort_unstable();
        vec.dedup();
        return Self { vec };
    }
}

impl<T: Ord> IntoIterator for SortedVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        return self.vec.into_iter();
    }
}

#[test]
fn sorted_vec_test() {
    let v: SortedVec<u32> = (0..1000).collect();
    assert_eq!(v.contains(&333), true);
    assert_eq!(v.contains(&1333), false);
}

impl<T: Ord> std::ops::Index<usize> for SortedVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        return self.vec.get(index).expect("Sorted Vec out of bounds error");
    }
}

impl<T: Ord> std::iter::FromIterator<T> for SortedVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        return Self::from(iter.into_iter().collect::<Vec<T>>());
    }
}
