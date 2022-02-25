App has two modes: voyage and overview

# Voyage

When voyaging, you have three options:

## Complete

Do the work of the current step and tell intents you completed it. This will move the voyage to the next leaf 
in the tree. This will mark the task as done which  


## Reduce

Triggers an editor with the current task as a parent and lets you write the steps to complete the parent.
Uses the editor view, but subset to just this node. After finished. sets the voyage to the first leaf node and
displays it.

# Overview

Display the tree with the root at top left, children to the right in a stack.

## Modal editor

### Normal Mode

Shows selected node. Also shows a current leaf.
H => Goes up to the parent while keeping the leaf still.
J => Moves the current leaf to the next leaf. If the leaf is not a child of the selected node. Find the next parent with
as close of a depth as possible.
K => Same as J but moves current leaf to previous leaf.
L => Moves to the child one depth down which contains the selected leaf.

Holding shift modifies above bindings to move a given task around. Whichever task is selected gets cycled within the
current parent.

Holding ctrl modifies above bindings to permute tasks at the current level. This will permute the unfinished tasks at the currently
selected level. 
**QUESTION: HOW SHOULD THIS WORK WITH LEAFS VS PARENT NODES**

If there is a voyage in the group of nodes being permuted, the operation changes
- If the currently selected task is after the voyage, only those
tasks which are also after the voyage are permuted. 
- If the task is before the voyage, all tasks are permuted and the line moves. This 
results in a previously completed task being moved to the uncompleted position.

**QUESTION: WHAT SHOULD HAPPEN IF MULTIPLE VOYAGES ARE IN THE SAME PERMUTATION GROUP**
**IDEA: MAYBE PERMUTE UP TO THE NEAREST VOYAGE**

Pressing i will enter edit mode where you can edit the node like a textbox.
Node color is selected by depth. This color sets the voyage scheme

### Edit Mode

A given node can be one of a few things:

1. A task. This is the basic unit of work and can be split into subtasks at any time.

A task may also have outputs which are stored. This should be elaborated on as I use the system more.
Maybe enable text presented to the user to be a template which can use previous stored outputs.
**IDEA: THIS COULD BE THE BASIS FOR THE MEMORY TOKEN THING**

2. A conditional jump. Asks the user a question and either jumps to another node, or continues on.

This is the basis for loops. References nodes by their id. Maybe s


 
