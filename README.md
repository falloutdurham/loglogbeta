 [LogLog-Beta and More: A New Algorithm for Cardinality Estimation Based on LogLog Counting](https://arxiv.org/abs/1612.02284)

 A new algorithm for estimating cardinalities. More efficient and easier to implement than your standard common or garden HyperLogLog. 

 Here's a Rust implementation.

 (some of the implementation code borrows liberally from Coda Hale's [Sketchy](https://github.com/codahale/sketchy) library)

 ```
 use loglogbeta::LogLogBeta;

 let mut llb = LogLogBeta::new(0.05); // 5% margin of error

 for i in 0..10000 {
     llb.insert(i);
 }
 assert!(llb.estimate() < 10500.0);
 assert!(llb.estimate() >  9500.0);
 ```
