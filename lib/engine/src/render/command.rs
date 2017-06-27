use cgmath::{Matrix4, Point2, Vector3};
use imgui::Ui;
use std::vec;
use std::borrow::Cow;

use camera::ComputedCamera;
use color::Color;

#[cfg_attr(feature = "cargo-clippy", allow(large_enum_variant))]
pub enum DrawCommand<Event> {
    Clear { color: Color },
    Points {
        buffer_name: Cow<'static, str>,
        size: f32,
        color: Color,
        model: Matrix4<f32>,
        camera: ComputedCamera,
    },
    Lines {
        buffer_name: Cow<'static, str>,
        width: f32,
        color: Color,
        model: Matrix4<f32>,
        camera: ComputedCamera,
    },
    Solid {
        buffer_name: Cow<'static, str>,
        light_dir: Vector3<f32>,
        color: Color,
        model: Matrix4<f32>,
        camera: ComputedCamera,
    },
    Text {
        font_name: Cow<'static, str>,
        color: Color,
        text: String,
        size: f32,
        position: Point2<f32>,
        screen_matrix: Matrix4<f32>,
    },
    Ui { run_ui: Box<Fn(&Ui) -> Vec<Event> + Send>, },
}

pub struct CommandList<Event> {
    commands: Vec<DrawCommand<Event>>,
}

impl<Event> CommandList<Event> {
    pub fn new() -> CommandList<Event> {
        CommandList { commands: Vec::new() }
    }

    pub fn clear(&mut self, color: Color) {
        self.commands.push(DrawCommand::Clear { color: color });
    }

    pub fn points<S>(
        &mut self,
        buffer_name: S,
        size: f32,
        color: Color,
        model: Matrix4<f32>,
        camera: ComputedCamera,
    ) where
        S: Into<Cow<'static, str>>,
    {
        self.commands.push(DrawCommand::Points {
            buffer_name: buffer_name.into(),
            size: size,
            color: color,
            model: model,
            camera: camera,
        });
    }

    pub fn lines<S>(
        &mut self,
        buffer_name: S,
        width: f32,
        color: Color,
        model: Matrix4<f32>,
        camera: ComputedCamera,
    ) where
        S: Into<Cow<'static, str>>,
    {
        self.commands.push(DrawCommand::Lines {
            buffer_name: buffer_name.into(),
            width: width,
            color: color,
            model: model,
            camera: camera,
        });
    }

    pub fn solid<S>(
        &mut self,
        buffer_name: S,
        light_dir: Vector3<f32>,
        color: Color,
        model: Matrix4<f32>,
        camera: ComputedCamera,
    ) where
        S: Into<Cow<'static, str>>,
    {
        self.commands.push(DrawCommand::Solid {
            buffer_name: buffer_name.into(),
            light_dir: light_dir,
            color: color,
            model: model,
            camera: camera,
        });
    }

    pub fn text<S>(
        &mut self,
        font_name: S,
        color: Color,
        text: String,
        size: f32,
        position: Point2<f32>,
        screen_matrix: Matrix4<f32>,
    ) where
        S: Into<Cow<'static, str>>,
    {
        self.commands.push(DrawCommand::Text {
            font_name: font_name.into(),
            color: color,
            text: text,
            size: size,
            position: position,
            screen_matrix: screen_matrix,
        });
    }

    pub fn ui<F>(&mut self, run_ui: F)
    where
        F: Fn(&Ui) -> Vec<Event> + Send + 'static,
    {
        self.commands.push(
            DrawCommand::Ui { run_ui: Box::new(run_ui) },
        );
    }
}

impl<Event> IntoIterator for CommandList<Event> {
    type Item = DrawCommand<Event>;
    type IntoIter = vec::IntoIter<DrawCommand<Event>>;

    fn into_iter(self) -> vec::IntoIter<DrawCommand<Event>> {
        self.commands.into_iter()
    }
}
