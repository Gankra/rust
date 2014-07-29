/********************************************
****************COMMON **********************
********************************************/


/// Base trait that all collections should inherit from
/// Due to the way the Rust's type system works, a perfectly generic
/// collection of T's can support very few operations. In particular, the
/// ability to store things that *don't* implement Eq, and the
/// distinction between value stores and key-value stores means
/// there's little to be made available here. Still, the few
/// operations that require absolutely none of these distinctions
/// might as well be pulled out into a common interface
pub trait Collection {
    /// Return the number of values stored in the collection
    fn len(&self) -> uint;
    
    /// Return true if the collection is empty
    fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// A Mutable Collection. See Collection for reasoning.
pub trait MutableCollection: Collection {
    /// Remove all elements from the collection
    fn clear(&mut self);

    /// Request the collection reserve enough space for `capacity` elements.
    /// By default a no-op, since no collection *needs* to support this, or directly
    /// obey the request. May over-allocate for amortization purposes. Does nothing
    /// if current capacity is already sufficient. 
    /// For many collections, it probably makes no sense (doubly-linked-list).
    /// Handy for array-based structures, to avoid multiple grows
    fn reserve(&mut self, capacity: uint) {}

    /// Like reserve, but will never over-allocate
    fn reserve_exact(&mut self, capacity: uint) {}

    /// Reserve space for `extra` additional elements more than the collection's len
    fn reserve_additional(&mut self, extra: uint) {
        self.reserve(self.len() + extra);
    }

    /// Request that the collection discard any unused capacity, see Request for reasoning
    fn shrink_to_fit(&mut self) {}
}








/********************************************
*************** CONTAINERS ******************
********************************************/


/// A Container, in contrast to a Map, is a direct value store.
/// All that an immutable container can generally due is report
/// all of its contents. Without Eq it is impossible to answer
/// even simple queries like "do you contain X". Containers are varied
/// enough that they need to be divided up into various sub-traits
/// to reasonably capture the whole landscape. 
///
/// This trait might be useful once generic iteration is available. For
/// now, a perfectly generic immutable container is basically useless.
/// Left in for posterity/clarity, at the moment. Effectively [deprecated].
pub trait Container<T>: Collection {} 

/// A Mutable version of Container. A Mutable container should always support
/// adding elements if it can exist at all. Having a mutable container by-value
/// should also permit the owner to move all of its contents.
pub trait MutableContainer<T>: Container<T> + MutableCollection {
    /// Insert a value into the Container in whatever way makes the most sense 
    /// by default. For a Stack this would be a push, for a queue this would be
    /// an enqueue. 
    /// Returns None if the container's len
    /// increased, or Some(x) if inserting the value requires x to be removed.
    /// For instance if a circular buffer is full it may choose to overwrite the
    /// oldest values, and will return them here. If inserting a duplicate into a
    /// Set, the original will be returned here.
    ///
    /// Possible use case: return the value to be inserted, if insertion can fail?
    fn swap (&mut self, value: T) -> Option<T>; //only really meaningful for Sets?

    /// Insert a value into the Container in whatever way makes the most sense 
    /// by default. For a Stack this would be a push, for a queue this would be
    /// an enqueue. 
    /// Returns true if the container's len grew as a result.
    /// Just a simpler "I don't care" version of swap.
    fn insert(&mut self, value: T) -> bool {
        self.swap(value).is_none()
    }

    /// Move all the contents of the iterator into this one.
    ///
    /// Just a convenience method to avoid boilerplate 
    fn insert_all <I: Iterator<T>> (&mut self, iter: I) {
        for x in iter {
            self.insert(x); false
        }
    }
}

/// Searchable containers support identifying if an element exists inside
/// of them. Many Unstructured Containers may not always be searchable.
/// For instance a Vec<T> is not searchable, but a Vec<T: Eq> is searchable.
/// This trait allows them to make that distinction.
pub trait SearchableContainer <T> : Container<T> {
    /// Return true if the given value is contained within the Container.
    fn contains(&self, value: &T) -> bool; 

    /// Return true if every element in the given iterator is contained
    /// within this one.
    ///
    /// Just another boilerplate utility method
    fn contains_all <'a, I: Iterator<&'a T>> (&self, iter: I) {
        let mut found_all = true; // default true for empty containers
        for x in iter {
            found_all = self.contains(x);
            if !found_all { break; }
        }
        found_all
    }
}

/// Mutable version of SearchableContainer. Since elements can be found,
/// elements can now be removed generically. This is likely very inefficient for
/// e.g. a Vec or DList, but sometimes you really need it. On the other hand
/// this is completely natural for e.g. Sets.
pub trait MutableSearchableContainer <T> : MutableContainer<T> {
    /// Remove one copy of the given value from the Container. Returns None if
    /// it wasn't found, or Some(x) if the value x was removed.
    fn pop(&mut self, value: &T) -> Option<T>; 

    /// Remove one copy of the given value from the Container. Returns true
    /// if the value was found.
    fn remove(&mut self, value: &T) -> bool {
        self.pop(value).is_some()
    }

    /// Remove *all* copies of the given value from the Container. Returns the
    /// number of copies removed. Effecively just `remove()` for Sets, but meaningful
    /// for containers that support duplicate values 
    fn erase(&mut self, value : &T) -> uint {
        let mut count = 0;
        while self.remove(value) {
            count += 1;
        }
        count
    }

    /// Remove every value found in the given iterator from this one. Only as many duplicate
    /// copies will be removed as there are duplicate copies in the provided iterator
    fn remove_all <'a, Iterator<&'a T>> (&mut self, iter: I) {
        for x in iter {
            self.remove(x);
        }
    }

    /// Remove every copy of every value found in the given iterator from this one.
    fn erase_all <'a, Iterator<&'a T>> (&mut self, iter: I) {
        for x in iter {
            self.erase(x);
        }    
    }

    /// Remove every value *not* found in the given iterator. Not sure on the value of this.
    /// Java's Collection has it, though.
    ///
    /// Writing a default for this would be possible but *horrendously* inefficient with the
    /// current interfaces I've written. Like, building a third container to hold
    /// all the elements that were found to be in self, but not in other, and then
    /// remove_all'ing that container. Maybe we want a remove-supporting iterator?
    fn retain_all <'a, Iterator<&'a T>> (&mut self, iter: I);
}

/// A Container that maintains an internal sorting of the elements, permitting various queries
/// to be made on the collection that wouldn't necessarily be reasonable on others.
/// A TreeSet, for instance would be a SortedSet. A HashSet wouldn't, and in fact a perfectly
/// generic HashSet has no notion of Ord, and so couldn't support this if it wanted to.
/// In theory a HashSet<T:Ord> could implement this interface, but it would be stupid. 
pub trait SortedContainer<T>: SearchableContainer<T> {
    /// Return the smallest element in the collection, determined by the collection's own
    /// ordering, or None if empty
    fn min <'a> (&'a self) -> Option<&'a T>;

    /// Return the largest element in the collection, determined by the collection's own
    /// ordering, or None if empty
    fn max <'a> (&'a self) -> Option<&'a T>;

    /// Return the largest element less than the given value, or None if no such value exists
    fn lower_bound_exclusive(&self, value: &T) -> Option<T>; // defaulted for partial ord?

    /// Return the largest element less than or equal to the given value, or None if no such value exists
    fn lower_bound_inclusive(&self, value: &T) -> Option<T>; // defaulted for partial ord?

    /// Return the smallest element greater than the given value, or None if no such value exists
    fn upper_bound_exclusive(&self, value: &T) -> Option<T>; // defaulted for partial ord?

    /// Return the largest element greater than or equal to the given value, or None if no such value exists
    fn upper_bound_inclusive(&self, value: &T) -> Option<T>; // defaulted for partial ord?
}

/// Mutable version of SortedContainer
pub trait MutableSortedContainer<T>: SortedContainer<T> + MutableSearchableContainer<T> {
    /// Remove and return the smallest element in the collection, or None if empty
    fn pop_min(&self) -> Option<T>; // not possible to default, since min borrows &self :(
    
    /// Remove and return the smallest element in the collection, or None if empty
    fn pop_max(&self) -> Option<T>; // not possible to default, since max borrows &self :(

    // Note: no remove variants for pop here, since unlike remove, 
    // the user does not fundamentally *know* the value already, making
    // them less interesting. Could go either way, though.
}

/// Lists are containers with an underlying indexing of the elements.
/// Vecs and DLists are the canonical Lists. 
///
/// Currently assuming that all in-bounds indices in a List are
/// Occupied, and "in-bounds" is always [0, len)
pub trait List<T> : Container<T> {
    /// Return the element at the given index or None if out of bounds
    fn get <'a> (&'a self, index: uint) -> Option<&'a T>;
}

/// Mutable version of a List. Provides no ways to change the size
/// to support fixed-size lists.
pub trait MutableList <T> : List<T> + MutableContainer<T> {
    /// Insert the value at the given index if it is in-bounds, 
    /// and return the value that previously occupied that index, or None otherwise
    fn swap_at (&mut self, index: uint, value: T) -> Option<T>;

    /// Insert the value at the given index if it is in-bounds, and return true if it was.
    fn insert_at (&mut self, index: uint, value: T) -> bool {
        self.swap_at(index, value).is_some()
    }

    /// Switch the two the values at the given indices
    fn switch (&mut self, a:uint, b:uint); // might be an `unsafe` default impl for this?
}

/// A List that's actually resizable
// I'm not a *huge* fan of inheriting Deque, because I'd expect a Deque to be *efficient* at
// all operations, but Vec's unshift is horribly slow.
pub trait ResizableList <T> : MutableList<T> + Deque<T> {
    // TODO: Splice?

    /// Remove the value at the given index if it is in-bounds, and return the value
    /// that occupied that location if it was in-bounds, or None otherwise. Decrements
    /// all larger indices.
    fn pop_at (&mut self, index: uint) -> Option<T>;

    /// Remove the value at the given index if it is in-bounds, and return true if it was. 
    /// Decrements all larger indices.
    fn remove_at (&mut self, index: uint) -> bool {
        self.pop_at(index).is_some()
    }

    /// Remove elements until the collection contains at most `len` elements
    /// Does nothing if already small enough.
    fn truncate (&mut self, len: uint) -> uint {
        let amount = self.len() - len;
        for i in range(0, amount) {
            self.pop_back();
        }

        if amount < 0 { 0 } else { amount }
    }

    // TODO: some grow_* methods, require clone. Different trait?
}

/// If a List is Searchable, then we can report the index of certain contents
pub trait SearchableList<T> : List<T> + SearchableContainer<T> {
    // both of these methods can be trivially defaulted if we guarantee that a List iterates
    // in increasing order of indices

    /// Return the first index the value is found at, or None if it is not contained
    fn index_of (&self, value: &T) -> Option<uint>;  

    /// Return the last index the value is found at, or None if it is not contained
    fn last_index_of (&self, value: &T) -> Option<uint>;

    // ith_index_of?
}

/// A Set is a container which guarantees that no value is duplicated. Consequently,
/// it is fundamentally just a SearchableContainer with an additional implementation detail.
/// Similar to Eq vs PartialEq. However, the current Set trait (and various concrete impls) 
/// in libcollections have some extra set operations that I have no particularly strong
/// feelings about, but are included anyway. Seems odd to only accept
/// Self, but it permits superior optimization for e.g. TreeSets, I guess.
///
/// These methods can all easily be written with a for loop and `contains()`
/// even for two perfectly generic sets of different types.
pub trait Set<T>: SearchableContainer<T> {
    fn is_disjoint(&self, other: &Self);
    fn is_subset(&self, other: &Self);
    fn is_superset(&self, other: &Self);

    fn difference(&self, other: &Self) -> Self;
    fn symmetric_difference(&self, other: &Self) -> Self;
    fn union(&self, other: &Self) -> Self;
    fn intersection(&self, other: &Self) -> Self;
}

/// A Set that's mutable. Again, just an internal implementation detail trait
/// over a MutableSearchableContainer 
pub trait MutableSet<T>: Set<T> + MutableSearchableContainer<T> {
    /// Make self the difference of self and other
    fn difference_with(&mut self, other: &Self);
    /// Make self the symmetric difference of self and other
    fn symmetric_difference_with(&mut self, other: &Self);
    /// Make self the intersection of self and other
    fn intersection_with(&mut self, other: &Self);

    // union_with is *really* just insert_all.
}

/// It's a Queue. Come on.
pub trait Queue<T> : MutableContainer<T> {
    fn enqueue (&mut self, value: T);
    fn dequeue (&mut self) -> Option<T>;
    fn peek <'a> (&'a self) -> Option<&'a T>;
}

//// It's a Stack, seriously.
pub trait Stack<T> : MutableContainer<T> {
    fn push (&mut self, value: T);
    fn pop (&mut self) -> Option<T>;
    fn top <'a> (&'a self) -> Option<&'a T>;
}

/// A Double-ended queue. Nothing more.
///
/// default impls of Stack<T> and Queue<T> for Deque when possible?
/// Deque is-a Stack and Queue?
pub trait Deque<T> : MutableContainer<T> { 
    fn front<'a>(&'a self) -> Option<&'a T>;
    fn front_mut<'a>(&'a mut self) -> Option<&'a mut T>;
    fn back<'a>(&'a self) -> Option<&'a T>;
    fn back_mut<'a>(&'a mut self) -> Option<&'a mut T>;
    fn push_front(&mut self, elt: T);
    fn push_back(&mut self, elt: T);
    fn pop_back(&mut self) -> Option<T>;
    fn pop_front(&mut self) -> Option<T>;
}

/// It's a PriorityQueue, man. What do you want from me???
/// Note that while it's superficially a Queue, it's not *really* a Queue.
///
/// Not really a *Sorted* container, nor necessarily searchable? Depends on
/// if we require on partial or total ord in comparators. 
pub trait PriorityQueue<T> : MutableContainer<T> { 
    fn enqueue (&mut self, value: T);
    fn dequeue (&mut self) -> Option<T>;
    fn peek <'a> (&'a self) -> Option<&'a T>;
}







/********************************************
***************** MAPS **********************
********************************************/


/// Maps, in contrast to Containers are a key-value store. A map does not care
/// about the values it contains, but does care about its keys. A given value
/// can occur many times as long as it is under different keys, but the same
/// key cannot occur more than once. 
///
/// Maps do not need to be subdivided as much as Containers, due to the fact that
/// they are fundamentally searchable and indexed by their keys. Consequently, a
/// Rust Map is basically what you'd expect in any other language
pub trait Map<K, V>: Collection {
    /// Return the value associated with the given key, or None if none exists
    fn find<'a>(&'a self, key: &K) -> Option<&'a V>;

    /// Return true if the key is in the Map
    fn contains_key(&self, key: &K) {
        self.find(key).is_some()
    }
}

/// Mutable version of a Map
pub trait MutableMap<K, V>: Map<K, V> + MutableCollection {
    /// Insert the given key-value pair into the Map, and return the value
    /// that was already under the key, or None otherwise
    ///
    /// Do we want to return the old key as well?
    fn swap(&mut self, k: K, v: V) -> Option<V>;

    /// Remove the given key and associated value from the Map, and return the
    /// value that was under the key, or None otherwise
    ///
    /// Do we want to return the key as well?
    fn pop(&mut self, k: &K) -> Option<V>;

    /// Insert the given key-value pair into the Map, and return true
    /// if the key was not already in the Map
    fn insert(&mut self, key: K, value: V) -> bool {
        self.swap(key, value).is_none()
    }

    /// Remove the given key and associated value from the Map, and return true
    /// if it existed
    fn remove(&mut self, key: &K) -> bool {
        self.pop(key).is_some()
    }

    /// Move all contents of the given iterator into this one
    fn insert_all <I:Iterator<K,V>> (&mut self, iter: I) {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }

    /// Remove all the keys found in the given iterator from this one
    fn remove_all <'a, I: Iterator<&'a K>> (&mut self, iter: I) {
        for key in iter {
            self.remove(key);
        }
    }

    /// Remove all the keys *not* found in the given iterator
    fn retain_all <'a, I: Iterator<&'a K>> (&mut self, iter: I); // Defaultable...?

    /// Find the value associated with the given key, and return it mutably
    fn find_mut<'a>(&'a mut self, key: &K) -> Option<&'a mut V>;
}

/// A SortedMap is a Map that maintains an underlying ordering of its contents
///
/// A Treemap is the canonical SortedMap. A HashMap is not Sorted (though if its
/// keys were Ord, one could hypothetically implement this-- that's dumb though)
///
/// See SortedContainer for method descriptions
pub trait SortedMap<K,V>: Map<K,V> {
    fn min <'a> (&'a self) -> Option<(&'a K, &'a V)>;
    fn max <'a> (&'a self) -> Option<(&'a K, &'a V)>;
    
    //ugh, and I guess mut variants of these in SortedMutableMap (value only) too??
    //defaulted if keys implement PartialEq?
    fn lower_bound_exclusive(&self, value: &K) -> Option<(&K, &V)>; 
    fn lower_bound_inclusive(&self, value: &K) -> Option<(&K, &V)>;
    fn upper_bound_exclusive(&self, value: &K) -> Option<(&K, &V)>;
    fn upper_bound_inclusive(&self, value: &K) -> Option<(&K, &V)>;
}

/// Mutable version of SortedMap, see MutableSortedContainer
pub trait SortedMutableMap<K,V> : SortedMap<K,V> + MutableMap<K,V> {
    fn pop_min(&mut self) -> Option<(K, V)>;
    fn pop_max(&mut self) -> Option<(K, V)>;
}