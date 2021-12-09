use crate::prelude::{Real, INF};
use std::iter;
use term_size;

/// ASCII plot of a sequence of positive values.
///
/// Draw each point as a column filled with '*'s up to the maximum height.
pub fn plot_vbars(values: &[Real], height: usize) {
    if let Some((w, _)) = term_size::dimensions() {
        if w >= values.len() {
            _plot_vbars_worker(values, height);
        } else {
            let n = (values.len() as Real / w as Real).ceil() as usize;
            let sample = sub_sample(values, n);
            _plot_vbars_worker(&sample, height);
        }
    } else {
        _plot_vbars_worker(values, height)
    }
}

fn _plot_vbars_worker(values: &[Real], height: usize) {
    if values.is_empty() {
        return;
    }
    let max = values.iter().cloned().fold(-INF, |x, y| x.max(y));
    let step = max / height as Real;

    for i in 0..height + 1 {
        let mut ln = String::with_capacity(values.len());
        let h = (height - i) as Real * step;
        for &x in values {
            ln.push(if x > h - 0.5 * step { '*' } else { ' ' });
        }
        println!("{}", ln);
    }
}

fn sub_sample(values: &[Real], n: usize) -> Vec<Real> {
    let size = values.len() as Real / n as Real;
    let mut result = Vec::with_capacity(size.ceil() as usize);
    
    for i in 0..size as usize {
        let psum: Real = values[i..i + n].iter().sum();
        result.push(psum / n as Real);
    } 

    if result.capacity() as Real > size {
        let psum: Real = result[size as usize..].iter().sum();
        result.push(psum / (result.capacity() - size as usize) as Real);
    }
    
    return result; 
}

/// ASCII plot of a sequence of positive values horizontally.
///
/// Draw each is a row filled with '*'s up to the maximum width.
pub fn plot_hbars(values: &[Real], width: usize) {
    if values.is_empty() {
        return;
    }
    let max = values.iter().cloned().fold(-INF, |x, y| x.max(y));
    let step = max / width as Real;

    for &x in values {
        let n = (x / step) as usize;
        let mut ln = String::with_capacity(n + 1);
        ln.push('|');
        ln.extend(iter::repeat('=').take(n));
        println!("{}", ln);
    }
}
