`RefCell<T>` gives reference-counted access to a `T`, dynamically maintaining aliasing xor mutability. This library provides
an equivalent type `SubSlice<'a,T>`, for reference-counted access to subsets of a slice `&'a [T]`.
This type tracks how many outstanding references exists *per index*, and ensures at runtime that there are either multiple shared references
to a particular index, or at most one mutable reference. It is not threadsafe.