//! Subscription type for recurring events.
//!
//! Subscriptions represent ongoing sources of messages, such as:
//! - Periodic timers (animations, auto-refresh)
//! - External event sources (file watchers, socket listeners)
//!
//! Unlike commands which run once, subscriptions continue producing
//! messages until cancelled or the program exits.
//!
//! # Example
//!
//! ```rust
//! use ferment::Sub;
//! use std::time::Duration;
//!
//! enum Msg {
//!     AnimationTick,
//!     AutoSave,
//! }
//!
//! // A subscription that ticks every 16ms for animation
//! let animation: Sub<Msg> = Sub::interval(
//!     "animation",
//!     Duration::from_millis(16),
//!     || Msg::AnimationTick,
//! );
//!
//! // Batch multiple subscriptions
//! let subs: Sub<Msg> = Sub::batch(vec![
//!     Sub::interval("animation", Duration::from_millis(16), || Msg::AnimationTick),
//!     Sub::interval("autosave", Duration::from_secs(30), || Msg::AutoSave),
//! ]);
//! ```

use std::time::Duration;

/// A subscription representing an ongoing source of messages.
///
/// Subscriptions are declared by the model and managed by the runtime.
/// The runtime calls each active subscription's generator function at
/// the appropriate interval or event.
pub struct Sub<M> {
    inner: SubInner<M>,
}

enum SubInner<M> {
    /// No subscription
    None,
    /// Periodic timer subscription
    Interval {
        /// Unique identifier for this subscription
        id: String,
        /// Time between messages
        interval: Duration,
        /// Function to generate messages
        msg_fn: Box<dyn Fn() -> M + Send>,
    },
    /// Batch of subscriptions
    Batch(Vec<Sub<M>>),
}

impl<M> Sub<M> {
    /// Create an empty subscription (no events).
    ///
    /// # Example
    ///
    /// ```rust
    /// use ferment::Sub;
    ///
    /// enum Msg {}
    ///
    /// let sub: Sub<Msg> = Sub::none();
    /// ```
    #[inline]
    pub fn none() -> Self {
        Self { inner: SubInner::None }
    }

    /// Create a periodic interval subscription.
    ///
    /// The message function is called at the specified interval,
    /// producing messages continuously until the subscription is removed.
    ///
    /// The `id` is used to track subscription identity across updates.
    /// Subscriptions with the same ID are considered the same subscription
    /// and won't restart when the model updates.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ferment::Sub;
    /// use std::time::Duration;
    ///
    /// enum Msg { Tick }
    ///
    /// let sub: Sub<Msg> = Sub::interval(
    ///     "my-timer",
    ///     Duration::from_millis(100),
    ///     || Msg::Tick,
    /// );
    /// ```
    pub fn interval<F>(id: impl Into<String>, interval: Duration, msg_fn: F) -> Self
    where
        F: Fn() -> M + Send + 'static,
    {
        Self { inner: SubInner::Interval { id: id.into(), interval, msg_fn: Box::new(msg_fn) } }
    }

    /// Create a subscription that fires every N milliseconds.
    ///
    /// Convenience wrapper around `Sub::interval`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ferment::Sub;
    ///
    /// enum Msg { Tick }
    ///
    /// let sub: Sub<Msg> = Sub::every_millis("tick", 100, || Msg::Tick);
    /// ```
    pub fn every_millis<F>(id: impl Into<String>, millis: u64, msg_fn: F) -> Self
    where
        F: Fn() -> M + Send + 'static,
    {
        Self::interval(id, Duration::from_millis(millis), msg_fn)
    }

    /// Create a subscription that fires every N seconds.
    ///
    /// Convenience wrapper around `Sub::interval`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ferment::Sub;
    ///
    /// enum Msg { AutoSave }
    ///
    /// let sub: Sub<Msg> = Sub::every_secs("autosave", 30, || Msg::AutoSave);
    /// ```
    pub fn every_secs<F>(id: impl Into<String>, secs: u64, msg_fn: F) -> Self
    where
        F: Fn() -> M + Send + 'static,
    {
        Self::interval(id, Duration::from_secs(secs), msg_fn)
    }

    /// Combine multiple subscriptions into one.
    ///
    /// All subscriptions in the batch will be active simultaneously.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ferment::Sub;
    /// use std::time::Duration;
    ///
    /// enum Msg { FastTick, SlowTick }
    ///
    /// let sub: Sub<Msg> = Sub::batch(vec![
    ///     Sub::every_millis("fast", 16, || Msg::FastTick),
    ///     Sub::every_secs("slow", 1, || Msg::SlowTick),
    /// ]);
    /// ```
    pub fn batch(subs: Vec<Sub<M>>) -> Self {
        // Filter out None subscriptions
        let subs: Vec<_> = subs.into_iter().filter(|s| !s.is_none()).collect();

        if subs.is_empty() {
            Self::none()
        } else if subs.len() == 1 {
            subs.into_iter().next().unwrap()
        } else {
            Self { inner: SubInner::Batch(subs) }
        }
    }

    /// Check if this is an empty subscription.
    pub fn is_none(&self) -> bool {
        matches!(self.inner, SubInner::None)
    }

    /// Transform the message type of this subscription.
    ///
    /// This is useful for composing subscriptions from child models.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ferment::Sub;
    /// use std::time::Duration;
    ///
    /// enum ChildMsg { Tick }
    /// enum ParentMsg { Child(ChildMsg) }
    ///
    /// let child_sub: Sub<ChildMsg> = Sub::every_millis("tick", 100, || ChildMsg::Tick);
    /// let parent_sub: Sub<ParentMsg> = child_sub.map(ParentMsg::Child);
    /// ```
    pub fn map<N, F>(self, f: F) -> Sub<N>
    where
        F: Fn(M) -> N + Send + Sync + Clone + 'static,
        M: 'static,
        N: 'static,
    {
        match self.inner {
            SubInner::None => Sub::none(),
            SubInner::Interval { id, interval, msg_fn } => {
                let f = f.clone();
                Sub::interval(id, interval, move || f(msg_fn()))
            },
            SubInner::Batch(subs) => {
                Sub::batch(subs.into_iter().map(|s| s.map(f.clone())).collect())
            },
        }
    }

    /// Extract subscription entries for the runtime.
    pub(crate) fn into_entries(self) -> Vec<SubEntry<M>> {
        match self.inner {
            SubInner::None => vec![],
            SubInner::Interval { id, interval, msg_fn } => {
                vec![SubEntry { id, interval, msg_fn }]
            },
            SubInner::Batch(subs) => subs.into_iter().flat_map(|s| s.into_entries()).collect(),
        }
    }
}

impl<M> Default for Sub<M> {
    fn default() -> Self {
        Self::none()
    }
}

impl<M> std::fmt::Debug for Sub<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            SubInner::None => write!(f, "Sub::None"),
            SubInner::Interval { id, interval, .. } => {
                write!(f, "Sub::Interval({:?}, {:?})", id, interval)
            },
            SubInner::Batch(subs) => write!(f, "Sub::Batch({} subs)", subs.len()),
        }
    }
}

/// Internal representation of a single subscription entry.
pub(crate) struct SubEntry<M> {
    /// Unique identifier
    pub id: String,
    /// Interval between firings
    pub interval: Duration,
    /// Message generator function
    pub msg_fn: Box<dyn Fn() -> M + Send>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum TestMsg {
        Tick,
        Other,
    }

    #[test]
    fn test_sub_none() {
        let sub: Sub<TestMsg> = Sub::none();
        assert!(sub.is_none());
        assert!(sub.into_entries().is_empty());
    }

    #[test]
    fn test_sub_interval() {
        let sub: Sub<TestMsg> = Sub::interval("test", Duration::from_millis(100), || TestMsg::Tick);
        assert!(!sub.is_none());

        let entries = sub.into_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "test");
        assert_eq!(entries[0].interval, Duration::from_millis(100));
        assert_eq!((entries[0].msg_fn)(), TestMsg::Tick);
    }

    #[test]
    fn test_sub_batch() {
        let sub: Sub<TestMsg> = Sub::batch(vec![
            Sub::interval("a", Duration::from_millis(100), || TestMsg::Tick),
            Sub::interval("b", Duration::from_millis(200), || TestMsg::Other),
        ]);

        let entries = sub.into_entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "a");
        assert_eq!(entries[1].id, "b");
    }

    #[test]
    fn test_sub_batch_filters_none() {
        let sub: Sub<TestMsg> = Sub::batch(vec![
            Sub::none(),
            Sub::interval("a", Duration::from_millis(100), || TestMsg::Tick),
            Sub::none(),
        ]);

        let entries = sub.into_entries();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_sub_map() {
        enum Child {
            Tick,
        }
        enum Parent {
            Child(Child),
        }

        let child_sub: Sub<Child> =
            Sub::interval("tick", Duration::from_millis(100), || Child::Tick);
        let parent_sub: Sub<Parent> = child_sub.map(Parent::Child);

        let entries = parent_sub.into_entries();
        assert_eq!(entries.len(), 1);
        match (entries[0].msg_fn)() {
            Parent::Child(Child::Tick) => {},
        }
    }

    #[test]
    fn test_convenience_methods() {
        let sub1: Sub<TestMsg> = Sub::every_millis("a", 100, || TestMsg::Tick);
        let sub2: Sub<TestMsg> = Sub::every_secs("b", 1, || TestMsg::Tick);

        let entries1 = sub1.into_entries();
        let entries2 = sub2.into_entries();

        assert_eq!(entries1[0].interval, Duration::from_millis(100));
        assert_eq!(entries2[0].interval, Duration::from_secs(1));
    }
}
