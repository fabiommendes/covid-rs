use super::tracker::Tracker;
use crate::{prelude::EpiModel, sim::Population};
use getset::CopyGetters;
use std::{
    fmt::{Debug, Display},
    iter,
    ops::Add,
};

/// The table tracker stores a table of values that grow row by row typically
/// storing the epidemiological state of the population. The state must have a
/// uniform type and usually corresponds to  counts of the number of agents in
/// each compartment.
///
/// The table can also grow column-wise. Empty values are stored as zeros
/// coerced from the value `zero = T::from(0_u8)`.
#[derive(Clone, Debug, Default, CopyGetters)]
pub struct TableTracker<T> {
    #[getset(get_copy = "pub")]
    nrows: usize,
    #[getset(get_copy = "pub")]
    ncols: usize,
    buffer: Vec<T>,
}

impl<T: Copy> TableTracker<T> {
    pub fn new(dims: usize) -> Self {
        TableTracker {
            ncols: dims,
            nrows: 0,
            buffer: vec![],
        }
    }

    #[inline(always)]
    fn _idx(&self, i: usize, j: usize) -> usize {
        i * self.ncols + j
    }

    /// Get the j-th component at the i-th time.
    pub fn get(&self, i: usize, j: usize) -> Option<T> {
        self.buffer.get(self._idx(i, j)).map(|x| *x)
    }

    /// Merge two epicurve trackers.
    ///
    /// Both elements must have the same number of rows to return Some(value),
    /// otherwise return None.
    pub fn merge(&self, other: &Self) -> Option<Self>
    where
        T: From<u8>,
    {
        if self.nrows != other.nrows {
            println!("{}, {}", self.nrows, other.nrows);
            return None;
        }

        let mut out = TableTracker {
            ncols: self.ncols + other.ncols,
            nrows: self.nrows,
            buffer: vec![0_u8.into(); self.buffer.len() + other.buffer.len()],
        };

        for i in 0..out.nrows {
            let k = self._idx(i, 0);
            let k_ = other._idx(i, 0);
            out.buffer
                .extend_from_slice(&self.buffer[k..k + self.ncols]);
            out.buffer
                .extend_from_slice(&other.buffer[k_..k_ + other.ncols]);
        }

        return Some(out);
    }

    /// Add new column to tracker from iterator.
    ///
    /// Column can be larger or smaller than the number of rows. In the latter
    /// case, fill with the last value if bfill=true or with zeros otherwise.
    pub fn add_column(&mut self, data: impl Iterator<Item = T>, bfill: bool) -> &mut Self
    where
        T: From<u8>,
    {
        let col = {
            let mut buf: Vec<T> = data.take(self.nrows).collect();
            let elem: T = if bfill {
                *buf.last().unwrap_or(&0_u8.into())
            } else {
                0_u8.into()
            };
            let n = self.nrows - buf.len();
            buf.extend(iter::repeat(elem).take(n));
            buf
        };

        let mut buffer = Vec::with_capacity(self.nrows * (self.ncols + 1));

        for i in 0..self.nrows {
            let k = self._idx(i, 0);
            buffer.extend_from_slice(&self.buffer[k..k + self.ncols]);
            buffer.push(col[i]);
        }
        self.ncols += 1;
        self.buffer = buffer;
        return self;
    }

    /// Add a new column to a copy of tracker.
    ///
    /// `bfill` has the same meaning as in add_column(). 
    pub fn with_column(&self, data: impl Iterator<Item = T>, bfill: bool) -> Self
    where
        T: From<u8>,
    {
        let mut new = self.clone();
        new.add_column(data, bfill);
        return new;
    }

    /// Create a new zeroed-counter adding a new empty cell to each column.
    pub fn step(&mut self)
    where
        T: From<u8>,
    {
        self.buffer.extend(vec![0_u8.into(); self.ncols].iter());
        self.nrows += 1;
    }

    /// Increment the counter at i-th column.
    pub fn incr(&mut self, i: usize)
    where
        T: From<u8> + Add<T, Output = T>,
    {
        if i < self.ncols {
            let k = self._idx(self.nrows - 1, i);
            self.buffer[k] = self.buffer[k] + 1_u8.into();
        }
    }

    /// Update epicurves from population. If step=true, creates a new row and
    /// count elements in each compartment of population. If step=false, it
    /// simply update the current tip, accumulating any past values.
    pub fn update<P>(&mut self, population: &P, step: bool)
    where
        T: From<u8> + Add<T, Output = T>,
        P: Population,
        P::State: EpiModel,
    {
        if step || self.nrows == 0 {
            self.step()
        }
        population.each_agent(&mut |_, state: &P::State| self.incr(state.index()));
    }

    /// Return the i-th row.
    pub fn row(&self, i: usize) -> Option<Vec<T>> {
        if i >= self.nrows {
            return None;
        } else {
            let k = self._idx(i, 0);
            return Some(self.buffer[k..k + self.ncols].into());
        }
    }

    /// Return the i-th column.
    pub fn col(&self, i: usize) -> Option<Vec<T>> {
        let mut vec = vec![];
        for k in 0..self.nrows {
            vec.push(self.buffer[k * self.ncols + i])
        }
        return Some(vec);
    }

    /// Return the last row or a vector of zeros.
    pub fn tip(&self) -> Vec<T>
    where
        T: From<u8>,
    {
        if let Some(row) = self.row(self.nrows - 1) {
            return row;
        }
        return vec![0_u8.into(); self.ncols];
    }

    /// Render epicurves as CSV data
    pub fn render_csv(&self, head: &str, sep: char) -> String
    where
        T: Display,
    {
        let mut data = head.to_string();

        for i in 0..self.nrows - 1 {
            data.push('\n');
            data.push_str(&format!("{}", self.buffer[self._idx(i, 0)]));
            for j in 1..self.ncols {
                let k = self._idx(i, j);
                data.push(sep);
                data.push_str(&format!("{}", self.buffer[k]));
            }
        }
        return data;
    }
}

impl<'a, P, T> Tracker<P> for TableTracker<T>
where
    P: Population,
    P::State: EpiModel,
    T: From<u8> + Display + Add<Output = T> + Copy + Debug + Default,
{
    fn track(&mut self, value: &P) {
        self.update(value, true);
    }
}

impl<T> From<Vec<T>> for TableTracker<T>
where
    T: From<usize>,
{
    fn from(data: Vec<T>) -> Self {
        TableTracker {
            ncols: 1,
            nrows: data.len(),
            buffer: data,
        }
    }
}

impl<T> From<&Vec<T>> for TableTracker<T>
where
    T: From<usize> + Clone,
{
    fn from(data: &Vec<T>) -> Self {
        TableTracker {
            ncols: 1,
            nrows: data.len(),
            buffer: data.clone(),
        }
    }
}
