use amethyst::assets::{Progress, ProgressCounter};

/// A newtype for a mutable ProgressCounter reference
///
/// This struct is useful when you want to pass progress counter to a function
/// as a mutable reference and load multiple assets using Loader.
///
/// Because we have a mutable reference in our function, we would move it
/// into Loader. We don't want this if we use loader multiple times because the
/// reference is already moved.
///
/// Normally we would wrap this into a RefCell but we can't in this case because
/// the borrowed cell value doesn't implement the progress trait.
///
/// Hence we need to create a new-type to satisfy our needs.
///
pub struct ProgressCounterMutRef<'a>(&'a mut ProgressCounter);

impl ProgressCounterMutRef<'_> {
    pub fn new(progress_counter: &mut ProgressCounter) -> ProgressCounterMutRef<'_> {
        ProgressCounterMutRef(progress_counter)
    }
}

impl<'a> Progress for &mut ProgressCounterMutRef<'a> {
    type Tracker = <&'a mut ProgressCounter as Progress>::Tracker;

    fn add_assets(&mut self, num: usize) {
        self.0.add_assets(num)
    }

    fn create_tracker(self) -> Self::Tracker {
        self.0.create_tracker()
    }
}
