extern crate siphasher;

use std::cmp::{max};
use std::hash::{Hash,Hasher};
use self::siphasher::sip::SipHasher;
use std::marker::PhantomData;

/// [LogLog-Beta and More: A New Algorithm for Cardinality Estimation Based on LogLog Counting](https://arxiv.org/abs/1612.02284)
///
/// A new algorithm for estimating cardinalities. More efficient and easier to implement than
/// your standard common or garden HyperLogLog.
///
/// (some of the implementation code borrows liberally from Coda Hale's [Sketchy](https://github.com/codahale/sketchy) library)
///
/// ```
/// use loglogbeta::LogLogBeta;
///
/// let mut hll = LogLogBeta::new(0.05); // 5% margin of error
///
/// for i in 0..10000 {
///     hll.insert(i);
/// }
/// assert!(hll.estimate() < 10500.0);
/// assert!(hll.estimate() >  9500.0);
/// ```
///

pub struct LogLogBeta<E> {
    alpha: f64,
    p: usize,
    msize: u64,
    m: Vec<u64>,
    marker: PhantomData<E>,
}

impl<E: Hash> LogLogBeta<E> {
    /// Returns a new `LogLogBeta` counter with the given margin of error.

    pub fn new(error: f64) -> LogLogBeta<E> {
        let p = (1.04 / error).powi(2).log2().ceil() as usize;
        LogLogBeta::<E> {
            alpha: alpha(p),
            p: p,
            msize: 1 << p,
            m: vec![0; 1 << p],
            marker: PhantomData,
        }
    }

    /// Inserts an element into the LLB

    pub fn insert(&mut self, e: E) {
        let mut h = SipHasher::new();
        e.hash(&mut h);
        let x = h.finish();
        let w = x >> self.p;
        let j = x & (self.msize - 1);
        let idx = j as usize;
        self.m[idx] = max(self.m[idx], rho(w, 64 - self.p as isize));
    }


    /// Obtain a cardinality estimate from the LogLogBeta counter

    pub fn estimate(&self) -> f64 { 
        let z = self.m.iter().filter(|&i| *i == 0).count();
        let m_s = self.msize as f64;
        let beta = beta(z);

        self.alpha * m_s * (m_s - (z as f64)) / (beta + self.inverse_sum())
    }
    
    // TODO: Merge two LLBs
    pub fn merge(&self, b: LogLogBeta<E>) -> LogLogBeta<E> {
        b    
    }

    fn inverse_sum(&self) -> f64 {
        //self.m_vec.each {|i| sum += 1 / (2 ^ @m_vec[i])}
        self.m.iter().fold(0.0, |acc, &x| acc + (1.0 / (1 << x) as f64))
    }
}

fn alpha(p: usize) -> f64 {
    match p {
        4 => 0.674,
        5 => 0.697,
        6 => 0.709,
        _ => 0.7213 / (1.0 + 1.078 / (1 << (p)) as f64)
    }
}

fn rho(w: u64, max_width: isize) -> u64 {
    let rho = max_width - (64 - w.leading_zeros() as isize) + 1;
    if rho <= 0 {
        panic!("w overflow: {}/{}", w, max_width);
    }
    rho as u64
}

fn beta(z: usize) -> f64 {
    let z = z as f64;
    let z_l = (z + 1.0).log2();
    -0.370393911 * z
    + 0.070471823 * z_l
    + 0.17393686 * z_l.powi(2) 
    + 0.16339839 * z_l.powi(3)
    - 0.09237745 * z_l.powi(4) 
    + 0.03738027 * z_l.powi(5)
    - 0.005384159 * z_l.powi(6)
    + 0.00042419 * z_l.powi(7)
}

#[cfg(test)]
mod test {
    use loglogbeta;
    #[test]
    fn insert() {
        let actual = 1000000.0;
        let p = 0.05;
        let mut hll = loglogbeta::LogLogBeta::new(p);
        for i in 0..actual as usize {
            hll.insert(i);
        }

        assert!(hll.estimate() > (actual - (actual * p)));
        assert!(hll.estimate() < (actual + (actual * p)));
    }
}

