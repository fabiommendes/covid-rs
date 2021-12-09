use crate::{prelude::EpiModel, sim::Population};
use getset::{CopyGetters, Getters};
use std::{
    fmt::{Debug, Display},
    iter,
    ops::Add,
};

/// A uniform tabular data structure that store values that grow row by row.
///
/// This is typically used for storing various variables with the epidemiological state of
/// the population.
///
/// The table can also grow column-wise. Empty values are stored as zeros coerced from the value `zero = T::from(0_u8)`.
#[derive(Clone, Debug, Default, CopyGetters, Getters)]
pub struct Table<T> {
    #[getset(get_copy = "pub")]
    nrows: usize,

    #[getset(get = "pub")]
    columns: Vec<String>,

    buffer: Vec<T>,
}

impl<T: Copy> Table<T> {
    /// Create a new empty table with the given number of columns.
    pub fn new(columns: Vec<String>) -> Self {
        Table {
            columns: columns,
            nrows: 0,
            buffer: vec![],
        }
    }

    #[inline(always)]
    pub fn ncols(&self) -> usize {
        return self.columns.len();
    }

    #[inline(always)]
    fn _idx(&self, i: usize, j: usize) -> usize {
        i * self.ncols() + j
    }

    #[inline(always)]
    fn _zero() -> T
    where
        T: From<u8>,
    {
        return T::from(0_u8);
    }

    /// Get the cell at the i-th row and j-th column.
    pub fn get(&self, i: usize, j: usize) -> Option<T> {
        self.buffer.get(self._idx(i, j)).map(|x| *x)
    }

    /// Merge two tables.
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

        let mut columns = self.columns.clone();
        columns.extend_from_slice(&other.columns);

        let mut out = Table {
            columns: columns,
            nrows: self.nrows,
            buffer: vec![0_u8.into(); self.buffer.len() + other.buffer.len()],
        };

        for i in 0..out.nrows {
            let k = self._idx(i, 0);
            let k_ = other._idx(i, 0);
            out.buffer
                .extend_from_slice(&self.buffer[k..k + self.ncols()]);
            out.buffer
                .extend_from_slice(&other.buffer[k_..k_ + other.ncols()]);
        }

        return Some(out);
    }

    /// Add new column from iterator.
    ///
    /// Input array can be larger or smaller than the number of rows. In the latter
    /// case, fill with the last value if bfill=true or with zeros otherwise.
    pub fn add_column(
        &mut self,
        name: &str,
        data: impl Iterator<Item = T>,
        bfill: bool,
    ) -> &mut Self
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

        let mut buffer = Vec::with_capacity(self.nrows * (self.ncols() + 1));

        for i in 0..self.nrows {
            let k = self._idx(i, 0);
            buffer.extend_from_slice(&self.buffer[k..k + self.ncols()]);
            buffer.push(col[i]);
        }
        self.columns.push(name.to_string());
        self.buffer = buffer;
        return self;
    }

    /// Create a new zeroed-counter adding a new empty cell to each column.
    pub fn push_empty(&mut self)
    where
        T: From<u8>,
    {
        self.push_value(Self::_zero())
    }

    /// Create a new zeroed-counter adding a new empty cell to each column.
    pub fn push_value(&mut self, value: T) {
        self.buffer.extend(vec![value; self.ncols()].iter());
        self.nrows += 1;
    }

    /// Increment the counter at i-th column.
    pub fn incr(&mut self, i: usize)
    where
        T: From<u8> + Add<T, Output = T>,
    {
        if i < self.ncols() {
            let k = self._idx(self.nrows - 1, i);
            self.buffer[k] = self.buffer[k] + 1_u8.into();
        }
    }

    /// Return the i-th row, if it exists.
    ///
    /// Data is copied at return site.
    pub fn row(&self, i: usize) -> Option<Vec<T>> {
        if i >= self.nrows {
            return None;
        } else {
            let k = self._idx(i, 0);
            return Some(self.buffer[k..k + self.ncols()].into());
        }
    }

    /// Return the i-th column, if it exists.
    ///
    /// Data is copied at return site.
    pub fn col(&self, i: usize) -> Option<Vec<T>> {
        let mut vec = vec![];
        for k in 0..self.nrows {
            vec.push(self.buffer[k * self.ncols() + i])
        }
        return Some(vec);
    }

    /// Return the last row or a vector of zeros, if table is empty.
    pub fn tip(&self) -> Vec<T>
    where
        T: From<u8>,
    {
        if let Some(row) = self.row(self.nrows - 1) {
            return row;
        }
        return vec![0_u8.into(); self.ncols()];
    }

    /// Render epicurves as CSV data
    pub fn render_csv(&self, sep: char) -> String
    where
        T: Display,
    {
        let mut data = self.columns.join(&sep.to_string());

        for i in 0..self.nrows - 1 {
            data.push('\n');
            data.push_str(&format!("{}", self.buffer[self._idx(i, 0)]));
            for j in 1..self.ncols() {
                let k = self._idx(i, j);
                data.push(sep);
                data.push_str(&format!("{}", self.buffer[k]));
            }
        }
        return data;
    }

    /// Update epidemic counts from population.
    ///
    /// If expand=true, creates a new row and count elements in each compartment of population.
    /// If expand=false, it simply update the current tip, accumulating any past values.
    pub fn count_epidemic_compartments<P>(&mut self, population: &P, expand: bool)
    where
        T: From<u8> + Add<T, Output = T>,
        P: Population,
        P::State: EpiModel,
    {
        if expand || self.nrows == 0 {
            self.push_empty()
        }
        population.each_agent(&mut |_, state: &P::State| self.incr(state.index()));
    }
}
