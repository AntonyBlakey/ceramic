use crate::{artist::Artist, commands::Commands, layout::*, window_data::WindowData};

pub fn new(
    direction: Direction,
    axis: Axis,
    ratio: f64,
    count: usize,
    child_a: Box<dyn Layout>,
    child_b: Box<dyn Layout>,
) -> Box<SplitLayout> {
    Box::new(SplitLayout {
        direction,
        axis,
        ratio,
        count,
        children: (child_a, child_b),
    })
}

pub struct SplitLayout {
    axis: Axis,
    direction: Direction,
    ratio: f64,
    count: usize,
    children: (Box<dyn Layout>, Box<dyn Layout>),
}

impl Layout for SplitLayout {
    fn layout(
        &self,
        rect: &Bounds,
        windows: Vec<WindowData>,
    ) -> (Vec<WindowData>, Vec<Box<dyn Artist>>) {
        if windows.is_empty() {
            return Default::default();
        }

        let mut rect_1 = *rect;
        let mut rect_2 = *rect;

        match self.axis {
            Axis::X => {
                rect_1.size.width = (rect.size.width as f64 * self.ratio).floor() as u16;
                rect_2.size.width = rect.size.width - rect_1.size.width;
                match self.direction {
                    Direction::Increasing => rect_2.origin.x = rect_1.max_x(),
                    Direction::Decreasing => rect_1.origin.x = rect_2.max_x(),
                }
            }
            Axis::Y => {
                rect_1.size.height = (rect.size.height as f64 * self.ratio).floor() as u16;
                rect_2.size.height = rect.size.height - rect_1.size.height;
                match self.direction {
                    Direction::Increasing => rect_2.origin.y = rect_1.max_y(),
                    Direction::Decreasing => rect_1.origin.y = rect_2.max_y(),
                }
            }
        }

        if windows.len() > self.count {
            let (w1, w2) = windows.split_at(self.count);
            let (mut new_windows, mut artists) = self.children.0.layout(&rect_1, w1.to_vec());
            let (mut new_windows_2, mut artists_2) = self.children.1.layout(&rect_2, w2.to_vec());
            new_windows.extend(new_windows_2.drain(0..));
            artists.extend(artists_2.drain(0..));
            (new_windows, artists)
        } else {
            self.children.0.layout(&rect_1, windows)
        }
    }
}

impl Commands for SplitLayout {
    fn get_commands(&self) -> Vec<String> {
        let c0 = self.children.0.get_commands();
        let c1 = self.children.1.get_commands();

        let mut result = Vec::with_capacity(c0.len() + c1.len() + 4);

        result.push(String::from("increase_count"));
        if self.count > 1 {
            result.push(String::from("decrease_count"));
        }
        if self.ratio < 0.9 {
            result.push(String::from("increase_ratio"));
        }
        if self.ratio > 0.1 {
            result.push(String::from("decrease_ratio"));
        }

        result.extend(c0.iter().map(|c| format!("0/{}", c)));
        result.extend(c1.iter().map(|c| format!("1/{}", c)));

        result
    }

    fn execute_command(&mut self, command: &str, args: &[&str]) -> bool {
        if command.starts_with("0/") {
            self.children.0.execute_command(command.split_at(2).1, args)
        } else if command.starts_with("1/") {
            self.children.1.execute_command(command.split_at(2).1, args)
        } else {
            match command {
                "increase_count" => self.count += 1,
                "decrease_count" if self.count > 1 => self.count -= 1,
                "increase_ratio" if self.ratio < 0.9 => self.ratio += 0.05,
                "decrease_ratio" if self.ratio > 0.1 => self.ratio -= 0.05,
                _ => (),
            }
            true
        }
    }
}
