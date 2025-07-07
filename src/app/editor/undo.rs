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

use critic_format::streamed::Block;
use leptos::logging::log;

use super::EditorBlock;

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
trait UnRe: Replay + Invert + Sized {
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
    BlockNotFound(usize),
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
            Self::BlockNotFound(x) => {
                write!(f, "Unable to find block with id {x}")
            }
            Self::OldStateInconsistent => {
                write!(f, "The current state is not consistent with the state that should exist to undo an action")
            }
            Self::NothingToReplay => {
                write!(f, "There is no action available to undo")
            }
        }
    }
}
impl std::error::Error for ReplayError {}

/// The different types of Undo/Redo
pub(super) enum UnReStep {
    /// Data inside a block has changed (on:change of an input field)
    DataChange(DataChange),
    /// Two Blocks were exchanged
    BlockSwap(BlockSwap),
    /// Any number of consecutive Blocks was exchanged for any other number of any other
    /// consecutive blocks.
    ///
    /// This serves the following cases:
    /// - a block was deleted
    /// - a block was inserted
    /// - a block was split into multiple blocks
    /// - two blocks were merged
    BlockChange(BlockChange),
}
impl UnReStep {
    pub fn new_data_change(
        logical_index: usize,
        old_inner_block: Block,
        new_inner_block: Block,
    ) -> Self {
        Self::DataChange(DataChange::new(
            logical_index,
            old_inner_block,
            new_inner_block,
        ))
    }
    pub fn new_block_change(
        physical_index_of_change: usize,
        old_blocks: Vec<EditorBlock>,
        new_blocks: Vec<EditorBlock>,
    ) -> Self {
        Self::BlockChange(BlockChange::new(
            physical_index_of_change,
            old_blocks,
            new_blocks,
        ))
    }
    pub fn new_deletion(physical_index_of_change: usize, block: EditorBlock) -> Self {
        Self::new_block_change(physical_index_of_change, vec![block], vec![])
    }
    pub fn new_insertion(physical_index_of_change: usize, block: EditorBlock) -> Self {
        Self::new_block_change(physical_index_of_change, vec![], vec![block])
    }
    pub fn new_swap(physical_index_1: usize, physical_index_2: usize) -> Self {
        Self::BlockSwap(BlockSwap::new(physical_index_1, physical_index_2))
    }
}
impl Replay for UnReStep {
    fn replay(&self, blocks: &mut Vec<EditorBlock>) -> Result<(), ReplayError> {
        match self {
            Self::DataChange(x) => x.replay(blocks),
            Self::BlockSwap(x) => x.replay(blocks),
            Self::BlockChange(x) => x.replay(blocks),
        }
    }
}
impl Invert for UnReStep {
    fn invert(self) -> Self {
        match self {
            Self::DataChange(x) => Self::DataChange(x.invert()),
            Self::BlockSwap(x) => Self::BlockSwap(x.invert()),
            Self::BlockChange(x) => Self::BlockChange(x.invert()),
        }
    }
}
impl UnRe for UnReStep {}

pub(super) struct UnReStack {
    undo_stack: Vec<UnReStep>,
    redo_stack: Vec<UnReStep>,
}
impl Default for UnReStack {
    fn default() -> Self {
        Self {
            undo_stack: vec![],
            redo_stack: vec![],
        }
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

struct DataChange {
    /// The (logical) id of the block that was changed
    id: usize,
    /// The block before the change
    old_inner: Block,
    /// The block after the change
    new_inner: Block,
}
impl DataChange {
    pub fn new(id: usize, old_inner: Block, new_inner: Block) -> Self {
        Self {
            id,
            old_inner,
            new_inner,
        }
    }
}
impl Invert for DataChange {
    fn invert(self) -> Self {
        Self {
            id: self.id,
            old_inner: self.new_inner,
            new_inner: self.old_inner,
        }
    }
}
impl Replay for DataChange {
    fn replay(&self, blocks: &mut Vec<EditorBlock>) -> Result<(), ReplayError> {
        let block_to_revert = blocks
            .iter_mut()
            .find(|blck| blck.id() == self.id)
            .ok_or(ReplayError::BlockNotFound(self.id))?;
        block_to_revert
            .overwrite_inner(&self.old_inner, &self.new_inner)
            .ok_or(ReplayError::OldStateInconsistent)?;
        Ok(())
    }
}
impl UnRe for DataChange {}

/// The two blocks given by their logical IDs were swapped.
struct BlockSwap {
    /// physical position of the first block
    first: usize,
    /// logical id of the second block
    second: usize,
}
impl BlockSwap {
    pub fn new(first: usize, second: usize) -> Self {
        Self { first, second }
    }
}
impl Invert for BlockSwap {
    fn invert(self) -> Self {
        Self {
            first: self.second,
            second: self.first,
        }
    }
}
impl Replay for BlockSwap {
    fn replay(&self, blocks: &mut Vec<EditorBlock>) -> Result<(), ReplayError> {
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

/// Any number of consecutive blocks was exchanged for any other number of consecutive blocks
struct BlockChange {
    /// Location in the block vector where the touched blocks start
    physical_index_of_change: usize,
    /// The blocks before the chnage
    old_blocks: Vec<EditorBlock>,
    /// The blocks after the change
    new_blocks: Vec<EditorBlock>,
}
impl BlockChange {
    pub fn new(
        physical_index_of_change: usize,
        old_blocks: Vec<EditorBlock>,
        new_blocks: Vec<EditorBlock>,
    ) -> Self {
        Self {
            physical_index_of_change,
            old_blocks,
            new_blocks,
        }
    }
}
impl Invert for BlockChange {
    fn invert(self) -> Self {
        Self {
            physical_index_of_change: self.physical_index_of_change,
            old_blocks: self.new_blocks,
            new_blocks: self.old_blocks,
        }
    }
}
impl Replay for BlockChange {
    fn replay(&self, blocks: &mut Vec<EditorBlock>) -> Result<(), ReplayError> {
        // make sure the next blocks after physical_index_of_change are the correct ones
        log!(
            "physical: {:?}, old: {:?}, new: {:?}",
            self.physical_index_of_change,
            self.old_blocks,
            self.new_blocks
        );
        if self.physical_index_of_change + self.old_blocks.len() > blocks.len() {
            return Err(ReplayError::OldStateInconsistent);
        };
        if &blocks
            [self.physical_index_of_change..self.physical_index_of_change + self.old_blocks.len()]
            != &self.old_blocks
        {
            return Err(ReplayError::OldStateInconsistent);
        };
        // exchange the blocks
        blocks.splice(
            self.physical_index_of_change..self.physical_index_of_change + self.old_blocks.len(),
            self.new_blocks.clone().into_iter().map(|b| b.into()),
        );
        Ok(())
    }
}
impl UnRe for BlockChange {}
