use coffee;

use coffee::{
    graphics::{Color, Frame, Image, Mesh, Point, Quad, Rectangle, Shape, Window, WindowSettings},
    load::Task,
    Game, Result, Timer,
};

use physical_ui::{nphysics_layer, NPhysicsLayer};

fn main() -> Result<()> {
    MyGame::run(WindowSettings {
        title: String::from("A caffeinated game"),
        size: (1280, 1024),
        resizable: true,
        fullscreen: false,
        maximized: false,
    })
}

struct MyGame {
    // Your game state and assets go here...
    //red: Image,
    layer: NPhysicsLayer<u32>,
    tick: u32,
}

impl Game for MyGame {
    type Input = (); // No input data
    type LoadingScreen = (); // No loading screen

    fn load(window: &Window) -> Task<MyGame> {
        Task::succeed(|| {
            use physical_ui::{Point, Size};
            let mut layer = nphysics_layer();
            layer.upsert(0, Size::new(110.0, 50.0), Point::new(320.0, 340.0));
            layer.upsert(1, Size::new(120.0, 40.0), Point::new(350.0, 370.0));
            layer.upsert(2, Size::new(80.0, 30.0), Point::new(340.0, 320.0));
            layer.upsert(3, Size::new(150.0, 80.0), Point::new(400.0, 300.0));

            layer.upsert(104, Size::new(40.0, 50.0), Point::new(600.0, 300.0));
            layer.upsert(105, Size::new(50.0, 40.0), Point::new(600.0, 300.0));
            layer.upsert(106, Size::new(80.0, 20.0), Point::new(600.0, 300.0));
            layer.upsert(107, Size::new(20.0, 80.0), Point::new(600.0, 300.0));

            let mut index = 200;

            fn spawn_100(
                layer: &mut NPhysicsLayer<u32>,
                base_x: f32,
                base_y: f32,
                index: &mut u32,
            ) {
                for i in 0u32..100 {
                    let x = (i % 10) as f32 * 10.0;
                    let y = (i / 10) as f32 * 10.0;
                    layer.upsert(
                        *index,
                        Size::new(50.0, 20.0),
                        Point::new(base_x + x, base_y + y),
                    );
                    *index += 1;
                }
            }
            spawn_100(&mut layer, 300.0, 500.0, &mut index);
            spawn_100(&mut layer, 500.0, 500.0, &mut index);
            MyGame { layer, tick: 0 }
        })
    }

    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        if (self.tick > 300) {
            self.layer.update(false);
        }
        if (self.tick % 180 == 0) {
            let x = (self.tick % 360 / 180) as f32 * 30.0;
            use physical_ui::{Point, Size};
            self.layer
                .upsert(107, Size::new(20.0 + x, 80.0), Point::new(600.0, 300.0));
        }
        // Clear the current frame
        frame.clear(Color::BLACK);

        // Draw your game here. Check out the `graphics` module!
        let mut mesh = Mesh::new();
        for (key, region) in self.layer.regions() {
            let (location, size) = region.rect();
            let rect = Rectangle {
                x: location.x - size.width * 0.5,
                y: location.y - size.height * 0.5,
                width: size.width,
                height: size.height,
            };
            let shape = Shape::Rectangle(rect);
            //let anchor = region.anchor();
            let r = (key % 100) as f32 / 100.0;
            let g = ((key / 100) % 4) as f32 / 5.0 + 0.2;
            let b = (100 - (key % 100)) as f32 / 100.0;
            mesh.stroke(shape, Color::new(r, g, b, 1.0), 1.0);
            mesh.draw(&mut frame.as_target());
        }
        self.tick += 1;
    }
}
