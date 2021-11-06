use render::mesh::{Object, MatrixStack};
use render::mesh::Transform;
use render::Painter;
use math::Mat4;

use crate::input::{InputState, Key};

pub struct App {
    pub painter: render::WebGlPainter,
    pub input: InputState,
    pub time: Time,
}

impl App {
    pub fn new(painter: render::WebGlPainter) -> Self {
        Self {
            painter,
            input: Default::default(),
            time: Default::default(),
        }
    }

    pub fn start(&mut self) {
        let camera = Mat4::new_unit()
            .translate_y(-1.75);
        self.painter.set_camera(camera);
    }

    pub fn update(&mut self) -> Result<(), anyhow::Error> {
        if self.handle_input() {
            let delta_y = 0.02 * (self.time.time * 0.015).sin();
            let camera = self.painter.camera
                .translate_y(delta_y);
            self.painter.set_camera(camera);
        }

        let start = -5.0;
        let mut x = 0.0;
        let length = 10.0;
        for _ in 0..21 {
            self.painter.paint_line((start + x, 0.0, -length/2.0).into(), (start + x, 0.0, length/2.0).into())?;
            self.painter.paint_line((-length / 2.0, 0.0, start + x).into(), (length/2.0, 0.0, start + x).into())?;
            x += 0.5;
        }

/*        let transform = Mat4::new_unit()
            .scale((2.0, 0.5, 1.0))
            .translate((0.0, 1.0, 0.0));
        let stack = MatrixStack::new()
            .push(Mat4::new_scale((2.0, 0.5, 1.0)))
            .push(Mat4::new_translate((0.0, 1.0, 0.0))).clone();
        let object_ref = self.painter.load(&Object {
            vertices: &render::mesh::QUAD,
            transform: stack,
        })?;
        self.painter.paint(object_ref)?;*/

        let roll_amp = 360.0;
        let roll_freq = 0.001;
        let pos_x = 3.0 * (roll_freq * self.time.time).sin();
        let rot_x = roll_amp * (roll_freq * self.time.time).sin();
        let stack = MatrixStack::new()
            .translate(-0.5, -0.5, -0.5)
            .scale(3.0, 0.5, 0.5)
            .rotate(rot_x, 0.0, 0.0)
            .translate(-1.0, 0.25, pos_x)
            .clone();
        let verts = &render::mesh::cube();
        let object_ref = self.painter.load(&Object {
            vertices: verts,
            transform: stack,
        })?;
        self.painter.paint(object_ref)?;

        let aspect = self.painter.dimensions.x / self.painter.dimensions.y;
        let perspective = render::projection::perspective2(50.0, aspect, 1.0, 200.0);

        let stack = MatrixStack::new()
            .scale(1.0, 1.0, 1.0)
            .translate(-1.0, 0.0, 0.0)
            .rotate(0.0, rot_x, 0.0)
            .translate(5.0, 0.0, -5.0)
            .push(self.painter.camera)
            .push(perspective)
            .clone();
        let verts = render::mesh::cube();
        let object_ref = self.painter.load(&Object {
            vertices: &verts,
            transform: stack
        })?;
        self.painter.paint_simple(object_ref)?;

        /*let object_ref = self.painter.load(&Object {
            vertices: &render::mesh::QUAD,
            transform: Transform {
                position: (5.0, 0.0, 0.0).into(),
                scale: (2.0, 0.5, 0.0).into(),
                rotation: (0.0, 0.0, 0.0).into(),
            },
        })?;
        self.painter.paint(object_ref)?;*/

        Ok(())
    }

    fn handle_input(&mut self) -> bool {
        let rot_speed = 0.16;
        let move_speed = 0.006;
        let mut is_moving = false;

        let mut movement = Mat4::new_unit();
        if self.input.is_down(Key::ArrowUp) {
            movement = movement.translate_z(move_speed * self.time.delta_time);
            is_moving = true;
        }
        if self.input.is_down(Key::ArrowDown) {
            movement = movement.translate_z(-move_speed * self.time.delta_time);
            is_moving = true;
        }
        if self.input.is_down(Key::ArrowLeft) {
            movement = movement.rotate_y(-rot_speed * self.time.delta_time);
            is_moving = true;
        }
        if self.input.is_down(Key::ArrowRight) {
            movement = movement.rotate_y(rot_speed * self.time.delta_time);
            is_moving = true;
        }
        if self.input.is_down(Key::W) {
            movement = movement.rotate_x(-rot_speed * self.time.delta_time);
        }
        if self.input.is_down(Key::S) {
            movement = movement.rotate_x(rot_speed * self.time.delta_time);
        }
        self.painter.set_camera(self.painter.camera * movement);
        is_moving
    }
}

#[derive(Debug, Default)]
pub struct Time {
    pub time: f32,
    pub delta_time: f32,
    prev_time: f32,
}

impl Time {
    pub fn set_time(&mut self, time: f32) {
        self.prev_time = self.time;
        self.time = time;
        self.delta_time = self.time - self.prev_time;
    }
}
