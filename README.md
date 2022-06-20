# cnquadtree

A cardinal neighbor quadtree Rust library. **Under heavy development. Do not use.**

Cnquadtree includes both fast neighbor finding, and fast point and region location. The library is based off two research papers with some modifications.

To guarantee Rust's memory safety, most of the mutable operations are implemented as methods in the **tree** struct rather than the **node** itself. However, it brings some advantages compared to a typical node-containing-node-pointers quadtree implementation.

The tree struct keeps track of node levels and the number of nodes in a level. This allows the use of efficient point and region location without setting a limit to the maximum level of the tree.

## Todo
* Complete main implementation
* Add tests
* Complete documentation (including readme)
* Add no-std support
* Add ghostcell implementation

## Research Papers
* [Simple and Efficient Traversal Methods for Quadtrees and Octrees](https://www.merl.com/publications/docs/TR2002-41.pdf) by Frisken & Perry (2002)
* [Cardinal Neighbor Quadtree: a New Quadtree-based
  Structure for Constant-Time Neighbor Finding](https://www.ijcaonline.org/research/volume132/number8/qasem-2015-ijca-907501.pdf) by Qasem & Touir (2015)

# License
Licensed under either of
* Apache License, Version 2.0
* MIT license

at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
