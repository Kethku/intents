use std::ops::{Index, IndexMut};

pub type TaskId = usize;

pub struct Step {
    pub prompt: String,
}

pub struct Task {
    pub id: TaskId,
    pub depth: usize,
    pub step: Step, 
    pub parent: Option<TaskId>,
    pub children: Vec<TaskId>
}

pub struct Map {
    pub tasks: Vec<Task>,
}

impl Index<TaskId> for Map {
    type Output = Task;

    fn index(&self, id: TaskId) -> &Self::Output {
        &self.tasks[id]
    }
}

impl IndexMut<TaskId> for Map {
    fn index_mut(&mut self, index: TaskId) -> &mut Self::Output {
        &mut self.tasks[index]
    }
}

impl Map {
    pub fn new(root_task: &str) -> Map {
        Map {
            tasks: vec![
                Task {
                    id: 0,
                    depth: 0,
                    step: Step {
                        prompt: root_task.to_string(),
                    },
                    parent: None,
                    children: vec![],
                }
            ],
        }
    }

    pub fn add_child(&mut self, parent: TaskId, new_prompt: &str) -> TaskId {
        let id = self.tasks.len();
        self.tasks[parent].children.push(id);
        self.tasks.push(Task {
            id,
            depth: self.tasks[parent].depth + 1,
            step: Step {
                prompt: new_prompt.to_string(),
            },
            parent: Some(parent),
            children: Vec::new(),
        });
        id
    }

    pub fn first_leaf(&self, task_id: TaskId) -> TaskId {
        if self.tasks[task_id].children.is_empty() {
            task_id
        } else {
            self.first_leaf(self.tasks[task_id].children[0])
        }
    }

    // Given a leaf task id, find the next leaf task after it.
    pub fn next_leaf(&self, task_id: TaskId) -> Option<TaskId> {
        // Find the first parent with a successor and return it's first leaf.
        let parent_id = self.tasks[task_id].parent?;
        let parent_children = &self.tasks[parent_id].children;
        let successor_id = parent_children.iter()
            .position(|&child_id| child_id == task_id)
            .and_then(|task_index| parent_children.get(task_index + 1))
            .copied();
        if let Some(successor_id) = successor_id {
            Some(self.first_leaf(successor_id))
        } else {
            self.next_leaf(parent_id)
        }
    }
}
