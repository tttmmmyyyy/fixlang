/*
Consider a function f : X -> Y.
There is a total ordering on X.

Given a sorted and unduplicated sequence xs = [x1, x2, ..., xn] of X, we calculate the list f(xs) = [f(x1), f(x2), ..., f(xn)] of Y.
Assume that the function f is costly, but linking two sequences of Y is cheap.

The sequence xs can be edited by inserting or deleting an element.
When xs is updated, we need to recalculate f(xs).
We want to decrease the cost of recalculation by using cached f(xs')'s where xs' is a subsequence of xs in a past calculation.
*/

pub enum Unit<X> {
    Cached(Vec<X>),
    NotCached(Vec<X>),
}

impl<X> Unit<X> {
    pub fn items(&self) -> Vec<X>
    where
        X: Clone,
    {
        match self {
            Unit::Cached(xs) => xs.clone(),
            Unit::NotCached(xs) => xs.clone(),
        }
    }

    pub fn is_cached(&self) -> bool {
        match self {
            Unit::Cached(_) => true,
            Unit::NotCached(_) => false,
        }
    }
}

const MAX_CACHE_LEN: usize = 32;

// Given a sequence of elements of X, decompose it segments, each of which is either cached or not.
pub fn split_into_units<X: Clone>(
    sequence: &[X], // sorted
    is_cached: impl Fn(&[X]) -> bool,
) -> Vec<Unit<X>> {
    fn search_longest_cached_subsequence<'a, Key>(
        sequence: &'a [Key],
        is_cached: impl Fn(&[Key]) -> bool,
    ) -> usize /* length */ {
        for len in (1..=MAX_CACHE_LEN).rev() {
            if len > sequence.len() {
                continue;
            }
            if is_cached(&sequence[..len]) {
                return len;
            }
        }
        0
    }

    let mut segments = vec![];
    let mut i = 0;
    loop {
        if i >= sequence.len() {
            break;
        }
        let cache_len = search_longest_cached_subsequence(&sequence[i..], &is_cached);
        if cache_len > 0 {
            segments.push(Unit::Cached(sequence[i..i + cache_len].to_vec()));
            i += cache_len;
        } else {
            let mut j = i + 1;
            loop {
                if j >= sequence.len() {
                    break;
                }
                if j - i >= MAX_CACHE_LEN {
                    break;
                }
                let cache_len_at_j = search_longest_cached_subsequence(&sequence[j..], &is_cached);
                if cache_len_at_j > 0 {
                    break;
                }
                j += 1;
            }
            segments.push(Unit::NotCached(sequence[i..j].to_vec()));
            i = j;
        }
    }

    segments
}
