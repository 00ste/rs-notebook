use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use std::usize;

struct Point {
    x: i32,
    y: i32,
    dx: i32,
    dy: i32,
}

struct Stroke {
    color: Color,
    width: f32,
    points: Vec<Point>,
}

struct Page {
    strokes: Vec<Stroke>,
}

fn pixels_to_stroke(pixel_buffer: &[sdl2::rect::Point], interval: u32, points: &mut Vec<Point>) {
    points.clear();
    let n = pixel_buffer.len();

    // Edge cases
    if n <= 2 {
        for pixel in pixel_buffer.iter() {
            points.push(Point {
                x: pixel.x,
                y: pixel.y,
                dx: 0,
                dy: 0,
            });
        }

        return;
    }

    // Save all points from the beginning at the given interval
    let mut i = 0;
    while i <= pixel_buffer.len() - 2 {
        points.push(Point {
            x: pixel_buffer[i].x,
            y: pixel_buffer[i].y,
            dx: (6.0 * (pixel_buffer[i+1].x - pixel_buffer[i].x) as f32).floor() as i32,
            dy: (6.0 * (pixel_buffer[i+1].y - pixel_buffer[i].y) as f32).floor() as i32,
        });

        i += interval as usize;
    }

    // Always save the last point and velocity
    points.push(Point {
        x: pixel_buffer[n-1].x,
        y: pixel_buffer[n-1].y,
        dx: pixel_buffer[n-1].x - pixel_buffer[n-2].x,
        dy: pixel_buffer[n-1].y - pixel_buffer[n-2].y,
    });
}

fn evaluate_stroke_points(stroke_points: &[Point], output: &mut Vec<sdl2::rect::Point>) {
    let mut i: usize = 0;
    while i < stroke_points.len() - 1 {
        // Calculate coefficients
        let x0 = sdl2::rect::FPoint::new(stroke_points[i].x    as f32, stroke_points[i].y    as f32);
        let x1 = sdl2::rect::FPoint::new(stroke_points[i+1].x  as f32, stroke_points[i+1].y  as f32);
        let v0 = sdl2::rect::FPoint::new(stroke_points[i].dx   as f32, stroke_points[i].dy   as f32);
        let v1 = sdl2::rect::FPoint::new(stroke_points[i+1].dx as f32, stroke_points[i+1].dy as f32);

        let p1 = sdl2::rect::FPoint::new(x0.x             , x0.y             );
        let p2 = sdl2::rect::FPoint::new(3.0 * x0.x + v0.x, 3.0 * x0.y + v0.y);
        let p3 = sdl2::rect::FPoint::new(3.0 * x1.x - v1.x, 3.0 * x1.y - v1.y);
        let p4 = sdl2::rect::FPoint::new(x1.x             , x1.y             );

        // Evaluate spline for many parameter values
        let mut t: f32 = 0.0;
        while t <= 1.0 {
            let u = 1.0 - t;
            output.push(sdl2::rect::Point::new(
                (t*t*t*p4.x + t*t*u*p3.x + t*u*u*p2.x + u*u*u*p1.x).floor() as i32,
                (t*t*t*p4.y + t*t*u*p3.y + t*u*u*p2.y + u*u*u*p1.y).floor() as i32,
            ));
            t += 0.15;
        }
        i += 1;
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let width = 1200;
    let height = 900;

    let window = video_subsystem.window("rust-sdl2 demo", width, height)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut pages: Vec<Page> = Vec::new();
    pages.push(Page { strokes: Vec::new() });
    let mut current_page = 0;
    let mut pixel_buffer: Vec<sdl2::rect::Point> = Vec::new();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'main: loop {
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, ..} => {
                    println!("started holding at {}, {}", x, y);
                },

                Event::MouseButtonUp { mouse_btn: MouseButton::Left, x, y, ..} => {
                    println!("stopped holding at {}, {}; saved {} points:", x, y, pixel_buffer.len());
                    for pixel in pixel_buffer.iter() {
                        println!("{}, {}", pixel.x, pixel.y);
                    }

                    // Init new stroke
                    pages[current_page].strokes.push(Stroke{
                        color: Color::BLACK,
                        width: 1.0,
                        points: Vec::new(),
                    });

                    // Convert the recorded raw pixels into a smaller set of stroke points and velocities
                    let n = pages[current_page].strokes.len() - 1;
                    pixels_to_stroke(&pixel_buffer, 5, &mut pages[current_page].strokes[n].points);

                    println!("Converted to stroke containing {} points:", pages[current_page].strokes[n].points.len());
                    for point in pages[current_page].strokes[n].points.iter() {
                        println!("x: {}, y: {}, dx: {}, dy: {}", point.x, point.y, point.dx, point.dy);
                    }

                    // Clear pixel buffer
                    pixel_buffer.clear();

                },
                _ => {}
            }
        }

        // Record new mouse position into temporary pixel buffer
        let mouse_state = sdl2::mouse::MouseState::new(&event_pump);
        if mouse_state.left() {
            let new_pixel = sdl2::rect::Point::new(mouse_state.x(), mouse_state.y());
            if pixel_buffer.len() == 0 {
                pixel_buffer.push(new_pixel);
            }
            else {
                let n = pixel_buffer.len() - 1;
                if pixel_buffer[n].x != new_pixel.x || pixel_buffer[n].y != new_pixel.y {
                    pixel_buffer.push(new_pixel);
                }
            }
        }

        // Render points on canvas
        canvas.set_draw_color(Color::WHITE);
        canvas.fill_rect(sdl2::rect::Rect::new(0, 0, 1200, 900)).unwrap();
        canvas.set_draw_color(Color::BLACK);
        canvas.draw_lines(pixel_buffer.as_slice()).unwrap();

        // Render strokes on canvas
        for stroke in pages[current_page].strokes.iter() {
            canvas.set_draw_color(stroke.color);
            let mut points_list: Vec<sdl2::rect::Point> = Vec::new();
            evaluate_stroke_points(&stroke.points, &mut points_list);
            canvas.draw_lines(points_list.as_slice()).unwrap();
        }

        /*
        // Render velocity vectors (debug)
        canvas.set_draw_color(Color::GREEN);
        for stroke in pages[current_page].strokes.iter() {
            for point in stroke.points.iter() {
                canvas.draw_line(
                    sdl2::rect::Point::new(point.x, point.y),
                    sdl2::rect::Point::new(point.x + point.dx, point.y + point.dy)
                ).unwrap();
            }
        }
        */

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
