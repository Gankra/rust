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
pub trait Container<T>: Collection {
    /// Call the provided closure on all of the Container's contents
    /// in whatever order the Container wants. Stops calling the
    /// closure after the first time it yields `true`
    fn foreach(&self, f: |&T| -> bool);
}

/// For containers that don't care about the values of their contents
/// The canonical example of an unstructured container is an indexed
/// container like a Deque or Vec. The canonical example of a structure
/// that *isn't* unstructured would be a Set, which must enforce
/// non-duplicates, or a Heap, which internally organizes its contents
/// to answer queries about them.
///
/// Unstructured containers can consequently permit their contents to
/// be mutated freely.
pub trait UnstructuredContainer<T> : MutableContainer<T> { // dumb name, at a loss
    /// Call the provided closure on all of the Container's contents
    /// in whatever order the Container wants, permiting mutations. 
    /// Stops calling the closure after the first time it yields `true`.
    fn foreach_mut(&mut self, f: |&mut T| -> bool); 
}

/// A Mutable version of Container. A Mutable container should always support
/// adding elements if it can exist at all. Having a mutable container by-value
/// should also permit the owner to move all of its contents.
pub trait MutableContainer<T>: Container<T> + MutableCollection {
    /// Call the provided closure on all of the Container's contents
    /// in whatever order the Container wants, moving the values into the
    /// closure. The Container cannot be used after this method is called. 
    /// Stops calling the closure after the first time it yields `true`.
    fn foreach_move(self, f:|T| -> bool);

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

    /// Move all the contents of another container into this one.
    ///
    /// Just a convenience method to avoid boilerplate 
    fn insert_all <C: MutableContainer> (&mut self, other: C) {
        other.foreach_move(|x| {
            self.insert(x); false
        });
    }
}

/// Searchable containers support identifying if an element exists inside
/// of them. Many Unstructured Containers may not always be searchable.
/// For instance a Vec<T> is not searchable, but a Vec<T: Eq> is searchable.
/// This trait allows them to make that distinction.
pub trait SearchableContainer <T> : Container<T> {
    /// Return true if the given value is contained within the Container.
    ///
    /// Should be able to default impl this for all SearchableContainers 
    /// that have a T that implements Eq (PartialEq?), using Container.foreach
    /// though this would break with today's trait system
    fn contains(&self, value: &T) -> bool; 

    /// Return true if every element in the given container is contained
    /// within this one.
    ///
    /// Just another boilerplate utility method
    fn contains_all <C: Container<T>> (&self, other: &C) {
        let mut found = true; // default true for empty containers
        other.foreach(|x| {
            found = self.contains(x);
            found
        });
        found
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

    /// Remove every value found in the given container from this one. Only as many duplicate
    /// copies will be removed as there are duplicate copies in the provided container
    fn remove_all <C: Container> (&mut self, other: &C) {
        other.foreach(|x| {
            self.remove(x); false
        });
    }

    /// Remove every copy of every value found in the given container from this one.
    fn erase_all <C: Container> (&mut self, other: &C) {
        other.foreach(|x| {
            self.erase(x); false
        });    
    }

    /// Remove every value *not* found in the given container. Not sure on the value of this.
    /// Java's Collection has it, though.
    ///
    /// Writing a default for this would be possible but *horrendously* inefficient with the
    /// current interfaces I've written. Like, building a third container to hold
    /// all the elements that were found to be in self, but not in other, and then
    /// remove_all'ing that container. Maybe we want a remove-supporting iterator?
    fn retain_all <C: Container> (&mut self, other: &C) ;
}

/// A Container that maintains an internal sorting of the elements, permiting various queries
/// to be made on the collection that wouldn't necessarily be reasonable on others.
/// A TreeSet, for instance would be a SortedSet. A HashSet wouldn't, and in fact a perfectly
/// generic HashSet has no notion of Ord, and so couldn't support this if it wanted to.
/// In theory a HashSet<T:Ord> could implement this interface, but it would be stupid. 
pub trait SortedContainer<T>: SearchableContainer<T> {
    // constructors for comparators?? Perhaps out of scope of traits.

    /// Call the provided closure on all of the Container's contents in the range [min, max]
    /// in the underlying sorted order of the Container. Stops calling the
    /// closure after the first time it yields `true`. 
    ///
    /// If either min or max is not provided, min and max will conceptually be replaced with
    /// -infinity and infinity respectively.
    ///
    /// Rather than Options for the bounds, it might be desirable to have a custom enum
    /// Bound <T> { Unbounded, Include(T), Exclude(T) }?
    fn foreach_sorted(&self, f: |&T| -> bool, min: Option<&T>, max: Option<&T>);

    /// Return the smallest element in the collection, determined by the collection's own
    /// ordering, or None if empty
    fn min <'a> (&'a self) -> Option<&'a T> {
        let mut min = None;
        self.foreach_sorted(|x| -> {
            min = Some(x); true
        });
        min
    }

    /// Return the largest element in the collection, determined by the collection's own
    /// ordering, or None if empty
    fn max <'a> (&'a self) -> Option<&'a T> {
        let mut max = None;
        self.foreach_sorted(|x| -> {
            max = Some(x); false
        });
        max
    }

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
    /// Call the provided closure on all of the Container's contents in the range [min, max]
    /// in the underlying sorted order of the Container. Stops calling the
    /// closure after the first time it yields `true`. 
    /// The Container cannot be used after this method is called. 
    /// Stops calling the closure after the first time it yields `true`.
    ///
    /// See SortedContainer.foreach_sorted for further details and thoughts
    /// Note that foreach_sorted_mut is not possible here, since SortedContainers
    /// are by definition structured.
    fn foreach_sorted_move(self, f:|T| -> bool, min: Option<&T>, max: Option<&T>);

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
/// These methods can all easily be written with for loop and `contains()`
/// even for two perfectly generic sets of different types.
pub trait Set<T>: SearchableContainer<T> {
    fn is_disjoint(&self, other: &Self) -> bool {
        let result = true;
        self.foreach(|x|{
            result = !result.contains(x); !result
        });
        result
    }

    fn is_subset(&self, other: &Self) -> bool {
        let result = true;
        other.foreach(|x|{
            result = self.contains(x); !result
        });
        result
    }; 

    fn is_superset(&self, other: &Self) -> bool {
        other.is_subset(self)
    };

    //need Default to default these
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
    /// Call the provided closure on all of the Map's contents
    /// in whatever order the Map wants. Stops calling the
    /// closure after the first time it yields `true`
    ///
    /// Do we want key-only/value-only variants? This doesn't seem
    /// *particularly* useful with tuples and destructuring.
    /// Maybe a handy convenience if we really don't care about one, or want
    /// to pipe directly into a Container?
    fn foreach(&self, f: |(&K, &V)| -> bool);

    /// Return the value associated with the given key, or None if none exists
    fn find<'a>(&'a self, key: &K) -> Option<&'a V>;

    /// Return true if the key is in the Map
    fn contains_key(&self, key: &K) {
        self.find(key).is_some()
    }
}

/// Mutable version of a Map
pub trait MutableMap<K, V>: Map<K, V> + MutableCollection {
    /// Call the provided closure on all of the Map's contents
    /// in whatever order the Map wants, providing mutable
    /// access to the values, since the Map doesn't care about them. 
    /// Stops calling the closure after the first time it yields `true`
    fn foreach_mut(&mut self, f:|(&K, &mut V)| -> bool);

    /// Call the provided closure on all of the Map's contents
    /// in whatever order the Map wants, moving the values into the
    /// closure. The Map cannot be used after this method is called. 
    /// Stops calling the closure after the first time it yields `true`.
    fn foreach_move(self, f:|(K, V)| -> bool);

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

    /// Move all contents of the given map into this one
    fn insert_all <M:MutableMap> (&mut self, other: M) {
        other.foreach_move(|(key, value)| {
            self.insert(key, value); false
        };
    }

    /// Remove all the keys found in the given map from this one
    fn remove_all <M:Map> (&mut self, other: &M) {
        other.foreach(|(key, _)|) {
            self.remove(key); false
        }
    }

    /// Remove all the keys *not* found in the given map
    fn retain_all <M:Map> (&mut self, other: &M); // Defaultable...?

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
    // constructors for comparators?? Possibly out of scope

    fn foreach_sorted(&self, f: |(&K, &V)| -> bool, min: Option<&K>, max: Option<&K>);

    fn min <'a> (&'a self) -> Option<(&'a K, &'a V)> {
        let min = None;
        self.foreach(|key, value| {
            min = Some((key, value)); true
        });
        min
    }

    fn max <'a> (&'a self) -> Option<(&'a K, &'a V)> {
        let max = None;
        self.foreach(|key, value| {
            max = Some((key, value)); false
        });
        max
    }
    
    //ugh, and I guess mut variants of these in SortedMutableMap (value only) too??
    //defaulted if keys implement PartialEq?
    fn lower_bound_exclusive(&self, value: &K) -> Option<(&K, &V)>; 
    fn lower_bound_inclusive(&self, value: &K) -> Option<(&K, &V)>;
    fn upper_bound_exclusive(&self, value: &K) -> Option<(&K, &V)>;
    fn upper_bound_inclusive(&self, value: &K) -> Option<(&K, &V)>;
}

/// Mutable version of SortedMap, see MutableSortedContainer
pub trait SortedMutableMap<K,V> : SortedMap<K,V> + MutableMap<K,V> {
    // We can mutate values, so we can have this too.
    fn foreach_sorted_mut(&mut self, f:|(&K, &mut V)| -> bool, min: Option<&K>, max: Option<&K>);
    fn foreach_sorted_move(self, f:|(K, V)| -> bool, min: Option<&K>, max: Option<&K>);

    fn pop_min(&mut self) -> Option<(K, V)>;
    fn pop_max(&mut self) -> Option<(K, V)>;
}