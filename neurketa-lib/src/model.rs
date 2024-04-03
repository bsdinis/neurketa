#![allow(unused)]
use std::cmp::Ordering;
use std::ops::{Add, Div, Mul, Sub};

/// Series of samples (useful for plotting CDF/PDFs)
pub struct SingleValueSeries<T: PartialOrd> {
    /// invariant: always sorted
    values: Vec<T>,
}

impl<T: PartialOrd> SingleValueSeries<T> {
    fn new() -> Self {
        SingleValueSeries { values: vec![] }
    }

    fn push(&mut self, v: T) {
        let pos = match self.values.binary_search_by(|x| x.partial_cmp(&v).unwrap()) {
            Ok(pos) => pos,
            Err(pos) => pos,
        };

        self.values.insert(pos, v);
    }
}

impl<T: PartialOrd> FromIterator<T> for SingleValueSeries<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut svs = SingleValueSeries::new();
        for v in iter {
            svs.push(v)
        }

        svs
    }
}

pub(crate) struct Event {
    pub(crate) delta: f64,
    pub(crate) depth: usize,
    pub(crate) id: String,
}

/// Series of timing events
pub struct TimeSeries {
    pub(crate) events: Vec<Event>,
}

impl TimeSeries {
    fn from_iter_and_offset<I: IntoIterator<Item = (std::time::Instant, usize, String)>>(
        orig_time: std::time::Instant,
        iter: I,
    ) -> Self {
        let events = iter
            .into_iter()
            .map(|(ts, depth, id)| Event {
                delta: (ts - orig_time).as_secs_f64(),
                depth,
                id,
            })
            .collect();

        TimeSeries { events }
    }
}

impl FromIterator<(f64, usize, String)> for TimeSeries {
    fn from_iter<I: IntoIterator<Item = (f64, usize, String)>>(iter: I) -> Self {
        let events = iter
            .into_iter()
            .map(|(delta, depth, id)| Event { delta, depth, id })
            .collect::<Vec<_>>();

        TimeSeries { events }
    }
}

/// Value obtained from a set of samples
pub struct StatisticalValue<T>
where
    T: Into<f64>
        + Copy
        + Clone
        + PartialOrd
        + Div<usize, Output = T>
        + for<'a> std::iter::Sum<&'a T>,
    for<'a, 'b> &'a T: Add<&'b T, Output = T> + Sub<&'b T, Output = T> + Mul<&'b T, Output = T>,
{
    samples: Vec<T>,
}

impl<T> StatisticalValue<T>
where
    T: Into<f64>
        + Copy
        + Clone
        + PartialOrd
        + Div<usize, Output = T>
        + for<'a> std::iter::Sum<&'a T>
        + std::iter::Sum<T>,
    for<'a, 'b> &'a T: Add<&'b T, Output = T> + Sub<&'b T, Output = T> + Mul<&'b T, Output = T>,
{
    fn new() -> Self {
        StatisticalValue { samples: vec![] }
    }

    fn push(&mut self, v: T) {
        let pos = match self
            .samples
            .binary_search_by(|x| x.partial_cmp(&v).unwrap())
        {
            Ok(pos) => pos,
            Err(pos) => pos,
        };

        self.samples.insert(pos, v);
    }

    /// panics on empty samples
    fn mean(&self) -> T {
        self.samples.iter().sum::<T>() / self.samples.len()
    }

    fn min(&self) -> T {
        *self.samples.first().unwrap()
    }

    fn max(&self) -> T {
        *self.samples.last().unwrap()
    }

    fn percentile(&self, p: f64) -> T {
        self.samples[(p * self.samples.len() as f64).ceil() as usize]
    }

    fn stddev(&self) -> f64 {
        fn cmp_fn<T: PartialOrd>(a: &T, b: &T) -> Ordering {
            a.partial_cmp(b).unwrap()
        }
        let mean = self.mean();

        let sum_of_diff_sq = self
            .samples
            .iter()
            .map(|v| std::cmp::max_by(v, &mean, cmp_fn) - std::cmp::min_by(v, &mean, cmp_fn))
            .map(|diff| &diff * &diff)
            .sum::<T>();

        let corrected_variance = (sum_of_diff_sq.into()) / ((self.samples.len() - 1) as f64);
        corrected_variance.sqrt()
    }
}

pub struct StatisticalSeries<X, T>
where
    X: Copy + Clone,
    T: Into<f64>
        + Copy
        + Clone
        + PartialOrd
        + Div<usize, Output = T>
        + for<'a> std::iter::Sum<&'a T>,
    for<'a, 'b> &'a T: Add<&'b T, Output = T> + Sub<&'b T, Output = T> + Mul<&'b T, Output = T>,
{
    series: Vec<(usize, X, StatisticalValue<T>)>,
}

impl<X, T> StatisticalSeries<X, T>
where
    X: Copy + Clone,
    T: Into<f64>
        + Copy
        + Clone
        + PartialOrd
        + Div<usize, Output = T>
        + for<'a> std::iter::Sum<&'a T>
        + std::iter::Sum<T>,
    for<'a, 'b> &'a T: Add<&'b T, Output = T> + Sub<&'b T, Output = T> + Mul<&'b T, Output = T>,
{
    fn new() -> Self {
        StatisticalSeries { series: vec![] }
    }

    fn push(&mut self, x: X, y: StatisticalValue<T>) {
        let max_order = self.series.last().map(|(order, _, _)| *order).unwrap_or(0);
        self.series.push((max_order + 1, x, y))
    }

    /// panics on empty samples
    fn mean(&self) -> Vec<(X, T)> {
        self.series
            .iter()
            .map(|(_order, x, y)| (*x, y.mean()))
            .collect()
    }

    fn min(&self) -> Vec<(X, T)> {
        self.series
            .iter()
            .map(|(_order, x, y)| (*x, y.min()))
            .collect()
    }

    fn max(&self) -> Vec<(X, T)> {
        self.series
            .iter()
            .map(|(_order, x, y)| (*x, y.max()))
            .collect()
    }

    fn percentile(&self, p: f64) -> Vec<(X, T)> {
        self.series
            .iter()
            .map(|(_order, x, y)| (*x, y.percentile(p)))
            .collect()
    }

    fn stddev(&self) -> Vec<(X, f64)> {
        self.series
            .iter()
            .map(|(_order, x, y)| (*x, y.stddev()))
            .collect()
    }
}
