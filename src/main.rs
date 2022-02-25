mod model;

use std::{
    any::Any,
    collections::HashMap,
};

use druid_shell::{
    kurbo::{Affine, BezPath, CubicBez, Point, Rect, Size, Vec2},
    piet::{
        Color, FontFamily, Piet, RenderContext, PietText, PietTextLayout, Text, 
        TextLayout, TextLayoutBuilder
    },
    Application, Region, WindowBuilder, WindowHandle, WinHandler,
};

use model::*;

const B1: Color = Color::rgb8(0x16, 0x20, 0x40);
const B2: Color = Color::rgb8(0x1a, 0x2a, 0x40);
const C: Color = Color::rgb8(0x31, 0x4a, 0x59);
const F2: Color = Color::rgb8(0x73, 0x62, 0x4d);
const F1: Color = Color::rgb8(0xbf, 0x8f, 0x64);

const BG_COLOR: Color = B1;
const NODE_BG_COLOR: Color = C;
const PARENT_CONNECTION_COLOR: Color = C;
const NEXT_TASK_CONNECTION_COLOR: Color = F2;
const TEXT_COLOR: Color = F1;
const FONT: FontFamily = FontFamily::SANS_SERIF;
const FONT_SIZE: f64 = 20.0;
const NODE_PADDING: f64 = 15.0;
const NODE_MARGIN: f64 = 30.0;
const OUTER_MARGIN: f64 = 35.0;
const CONNECTION_HANDLE_DISTANCE: f64 = 15.0;
const CONNECTION_WIDTH: f64 = 2.0;
const SELECTED_TASK_WIDTH: f64 = 4.0;
const SELECTED_TASK_COLOR: Color = F1;
const SELECTED_LEAF_COLOR: Color = B2;
const VOYAGE_MARKER_COLOR: Color = F1;

fn connect_to_parent(parent: Rect, child: Rect) -> CubicBez {
    let parent_center = parent.center();
    let parent_point = Point::new(parent.max_x(), parent_center.y);
    let parent_control = parent_point + Vec2::new(CONNECTION_HANDLE_DISTANCE, 0.0);
    let child_center = child.center();
    let child_point = Point::new(child.min_x(), child_center.y);
    let child_control = child_point - Vec2::new(CONNECTION_HANDLE_DISTANCE, 0.0);

    CubicBez::new(parent_point, parent_control, child_control, child_point)
}

fn connect_to_next(first: Rect, next: Rect) -> CubicBez {
    let first_center = first.center();
    let first_point = Point::new(first_center.x, first.max_y());
    let first_control = first_point + Vec2::new(0.0, CONNECTION_HANDLE_DISTANCE);
    let next_center = next.center();
    let next_point = Point::new(next_center.x, next.min_y());
    let next_control = next_point - Vec2::new(0.0, CONNECTION_HANDLE_DISTANCE);

    CubicBez::new(first_point, first_control, next_control, next_point)
}

struct Intents {
    handle: WindowHandle,
    size: Size,
    current_map: Map,
    text_engine: Option<PietText>,
    task_text_layouts: HashMap<TaskId, PietTextLayout>,
    task_locations: HashMap<TaskId, Rect>,
    selected_task: TaskId,
    voyage_position: TaskId,
}

impl Intents {
    fn draw_task(&mut self, task_id: usize, parent_rect: Option<Rect>, offset: Point, piet: &mut Piet) -> Rect {
        // Layout Text
        let task = &self.current_map.tasks[task_id];
        let text_layout = self.task_text_layouts.entry(task_id).or_insert_with(|| {
            self.text_engine.as_mut().unwrap()
                .new_text_layout(task.step.prompt.clone())
                .font(FONT, FONT_SIZE)
                .text_color(TEXT_COLOR)
                .build()
                .unwrap()
        });

        // Fill Background
        let text_rect = text_layout
            .image_bounds();

        let background_rect = text_rect
            .inflate(NODE_PADDING, NODE_PADDING)
            .with_origin(offset);
        piet.fill(background_rect, &NODE_BG_COLOR);

        // Draw Text
        let metric = text_layout.line_metric(0).unwrap();
        piet.draw_text(text_layout, offset + (NODE_PADDING, NODE_PADDING - metric.baseline + text_rect.height()));

        // Draw Parent Connection
        if let Some(parent_rect) = parent_rect {
            let connection = connect_to_parent(parent_rect, background_rect);
            piet.stroke(connection, &PARENT_CONNECTION_COLOR, CONNECTION_WIDTH);
        }

        self.task_locations.insert(task_id, background_rect);

        background_rect
    }

    // Draw the current task, and recursively draw its children returning the total bounding
    // box of rendered space without padding.
    fn draw_task_with_dependencies(&mut self, task_id: usize, parent_rect: Option<Rect>, offset: Point, piet: &mut Piet) -> Rect {
        // Call draw_task with self
        let self_rect = self.draw_task(task_id, parent_rect, offset, piet);
        let mut with_children_rect = self_rect.clone();

        // Loop over children and call draw_task_with_dependencies with child and new offset
        // adjusting the resulting offset
        let child_x = with_children_rect.max_x() + NODE_MARGIN;
        let mut child_y = with_children_rect.min_y();
        let child_ids = &self.current_map.tasks[task_id].children.clone();
        for child_id in child_ids {
            let child_bounding_box = self.draw_task_with_dependencies(
                *child_id,
                Some(self_rect),
                (child_x, child_y).into(), 
                piet);

            with_children_rect = with_children_rect.union(child_bounding_box);
            child_y = with_children_rect.max_y() + NODE_MARGIN;
        }

        // Return the combined bounding box
        with_children_rect
    }

    fn draw_task_path(&self, piet: &mut Piet) {
        let mut previous_task = self.current_map.first_leaf(0);
        while let Some(next_task) = self.current_map.next_leaf(previous_task) {
            let previous_rect = self.task_locations.get(&previous_task).unwrap().clone();
            let next_rect = self.task_locations.get(&next_task).unwrap().clone();

            let connection = connect_to_next(previous_rect, next_rect);
            piet.stroke(connection, &NEXT_TASK_CONNECTION_COLOR, 4.0);

            previous_task = next_task;
        }
    }

    fn draw_voyage_marker(&self, piet: &mut Piet) {
        let scale = Affine::scale(0.1);
        let voyage_task_position = self.task_locations.get(&self.voyage_position).unwrap();
        let translation = Affine::translate(voyage_task_position.origin().to_vec2() + voyage_task_position.size().to_vec2());
        let offset = Affine::translate(Vec2::new(-20.0, -30.0));
        let transform = translation * offset * scale;

        let first = BezPath::from_svg("M 83.805 321.065 L 137.599 385.051 L 364.666 384.485 L 434.881 318.8 L 83.805 321.065 Z").unwrap();
        piet.fill(transform * first, &VOYAGE_MARKER_COLOR);
        let second = BezPath::from_svg("M 280.294 44.168 L 277.463 304.077 L 129.105 306.342 L 280.294 44.168 Z").unwrap();
        piet.fill(transform * second, &VOYAGE_MARKER_COLOR);
        let third = BezPath::from_svg("M 291.574 94.564 L 289.921 304.076 L 336.92 303.498 L 291.574 94.564 Z").unwrap();
        piet.fill(transform * third, &VOYAGE_MARKER_COLOR);
    }
}

impl WinHandler for Intents {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = handle.clone();
        self.text_engine = Some(handle.text());
    }

    fn prepare_paint(&mut self) {
        self.handle.invalidate();
    }

    fn paint(&mut self, piet: &mut Piet, _: &Region) {
        let background_rect = self.size.to_rect();
        piet.fill(background_rect, &BG_COLOR);

        self.draw_task_with_dependencies(0, None, (OUTER_MARGIN, OUTER_MARGIN).into(), piet);

        self.draw_task_path(piet);

        let selected_rect = self.task_locations.get(&self.selected_task).unwrap().clone();
        piet.stroke(selected_rect, &SELECTED_TASK_COLOR, SELECTED_TASK_WIDTH);

        let selected_leaf = self.current_map.first_leaf(self.selected_task);
        let selected_leaf_rect = self.task_locations.get(&selected_leaf).unwrap().clone();
        piet.stroke(selected_leaf_rect, &SELECTED_LEAF_COLOR, SELECTED_TASK_WIDTH);

        self.draw_voyage_marker(piet);
    }

    fn size(&mut self, size: Size) {
        self.size = size;
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn request_close(&mut self) {
        self.handle.close();
    }

    fn destroy(&mut self) {
        Application::global().quit()
    }
}

fn main() {
    let mut map = Map::new("Hello world task");
    map.add_child(0, "Task1");
    let task_2 = map.add_child(0, "Task2");
    map.add_child(0, "Task3");

    map.add_child(task_2, "Task4");
    map.add_child(task_2, "Task5");
    map.add_child(task_2, "Task6");

    let voyage_position = map.first_leaf(0);

    let intents = Intents {
        size: Default::default(),
        handle: Default::default(),
        current_map: map,
        text_engine: None,
        task_text_layouts: HashMap::new(),
        task_locations: HashMap::new(),
        selected_task: 0,
        voyage_position,
    };

    let app = Application::new().unwrap();

    let mut builder = WindowBuilder::new(app.clone());
    builder.set_handler(Box::new(intents));
    builder.set_title("Intents");

    let window = builder.build().unwrap();
    window.show();

    app.run(None);
}
