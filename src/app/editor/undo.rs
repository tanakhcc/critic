//! The primitives used in the undo/redo scheme
//!
//! This undo mechanism is action-based.
//! Primarily, there is a Stack of [`UnReStep`]s; each of these steps records an action taken by
//! giving the relevant part of the state before and after the step.
//! Undo happens by replaying the inverse.
//!
//! Similarly, each undo pushes the inverse into the Redo-Stack. Redo pops and applies the last
//! element of the Redo-Stack.
//!
//! Doing anything other then an Undo/Redo clears the Redo-Stack. There is no Undo-Tree in this
//! editor.

use leptos::logging::log;

use super::{EditorBlock, InnerBlockDry};

/// Replayable thing in the stack machine.
trait Replay {
    /// Replay this action; taking old_state to new_state
    fn replay(&self, blocks: &mut Vec<EditorBlock>) -> Result<(), ReplayError>;
}
trait Invert {
    /// invert the effect of this Action
    fn invert(self) -> Self;
}
/// This trait is used to actually undo the action performed.
trait UnRe : Replay + Invert + Sized {
    /// Undo the effect by inverting it and then applying this inversion
    ///
    /// Also return the inverted effect for pushing it on the redo stack
    fn undo(self, blocks: &mut Vec<EditorBlock>) -> Result<Self, ReplayError> {
        let inverse = self.invert();
        inverse.replay(blocks)?;
        Ok(inverse)
    }
}

/// The different things that can go wrong while replaying
#[derive(Debug, PartialEq)]
pub enum ReplayError {
    /// The block with this id was not found
    ///
    /// This should never happen and is always a programmer error.
    BlockNotFound(i32),
    /// There was some way in which the current state is not the old state expected from the action
    /// to undo
    ///
    /// This should never happen and is always a programmer error.
    OldStateInconsistent,
    /// There is no action available to undo
    ///
    /// This is a user error.
    NothingToReplay,
}
impl core::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::BlockNotFound(x) => { write!(f, "Unable to find block with id {x}") }
            Self::OldStateInconsistent => { write!(f, "The current state is not consistent with the state that should exist to undo an action") }
            Self::NothingToReplay => { write!(f, "There is no action available to undo") }
        }
    }
}
impl std::error::Error for ReplayError {}

/// The different types of Undo/Redo
pub(super) enum UnReStep {
    DataChange(DataChange),
    BlockSwap(BlockSwap),
}
impl Replay for UnReStep {
    fn replay(&self, blocks: &mut Vec<EditorBlock>) -> Result<(), ReplayError> {
        match self {
            Self::DataChange(x) => Ok(x.replay(blocks)?),
            Self::BlockSwap(x) => x.replay(blocks),
        }
    }
}
impl Invert for UnReStep {
    fn invert(self) -> Self {
        match self {
            Self::DataChange(x) => Self::DataChange(x.invert()),
            Self::BlockSwap(x) => Self::BlockSwap(x.invert()),
        }
    }
}
impl UnRe for UnReStep {}

struct DataChange {
    /// The (logical) id of the block that was changed
    id: i32,
    /// The block before the change
    old_inner: InnerBlockDry,
    /// The block after the change
    new_inner: InnerBlockDry,
}
impl Invert for DataChange {
    fn invert(self) -> Self {
        Self { id: self.id, old_inner: self.new_inner, new_inner: self.old_inner }
    }
}
impl Replay for DataChange {
    fn replay(&self, blocks: &mut Vec<EditorBlock>) -> Result<(), ReplayError> {
        let block_to_revert = blocks.iter_mut().find(|blck| blck.id() == self.id).ok_or(ReplayError::BlockNotFound(self.id))?;
        block_to_revert.overwrite_inner(&self.old_inner, &self.new_inner).ok_or(ReplayError::OldStateInconsistent)?;
        Ok(())
    }
}
impl UnRe for DataChange {}

/// The two blocks given by their logical IDs were swapped.
pub(super) struct BlockSwap {
    /// physical position of the first block
    first: usize,
    /// logical id of the second block
    second: usize,
}
impl BlockSwap {
    pub fn new(first: usize, second: usize) -> Self {
        Self { first, second, }
    }
}
impl Invert for BlockSwap {
    fn invert(self) -> Self {
        Self { first: self.second, second: self.first }
    }
}
impl Replay for BlockSwap {
    fn replay(&self, blocks: &mut Vec<EditorBlock>) -> Result<(), ReplayError> {
        log!("Now replaying an action of type BlockSwap");
        // if the indices are the same, this is a noop
        if self.first == self.second {
            return Ok(());
        };
        if self.first >= blocks.len() || self.second >= blocks.len() {
            Err(ReplayError::OldStateInconsistent)
        } else {
            blocks.swap(self.first, self.second);
            Ok(())
        }
    }
}
impl UnRe for BlockSwap {}

pub(super) struct UnReStack {
    undo_stack: Vec<UnReStep>,
    redo_stack: Vec<UnReStep>,
}
impl Default for UnReStack {
    fn default() -> Self {
        Self { undo_stack: vec![], redo_stack: vec![], }
    }
}
impl UnReStack {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new undo-task
    ///
    /// Note: this clears the Redo-stack
    pub fn push_undo(&mut self, action: UnReStep) {
        // pushing a new undo always clears the redo stack
        self.redo_stack.clear();
        self.undo_stack.push(action);
    }

    /// Return true iff the next call to undo will perform an action
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Perform one undo step
    ///
    /// Returns Some(()) when a step was actually performed
    pub fn undo(&mut self, blocks: &mut Vec<EditorBlock>) -> Result<(), ReplayError> { 
        // pop from the undo stack
        let top_action = self.undo_stack.pop().ok_or(ReplayError::NothingToReplay)?;
        // undo
        let inverted = top_action.undo(blocks)?;
        // push to the redo stack
        self.redo_stack.push(inverted);
        Ok(())
    }

    /// Return true iff the next call to redo will perform an action
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Perform one redo step
    ///
    /// Returns Some(()) when a step was actually performed
    pub fn redo(&mut self, blocks: &mut Vec<EditorBlock>) -> Result<(), ReplayError> { 
        // pop from the redo stack
        let top_action = self.redo_stack.pop().ok_or(ReplayError::NothingToReplay)?;
        // redo
        let inverted = top_action.undo(blocks)?;
        // push to the redo stack
        self.undo_stack.push(inverted);
        Ok(())
    }
}

