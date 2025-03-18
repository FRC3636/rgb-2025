use std::{
    sync::{
        mpsc::{Receiver, Sender}, Arc, Barrier, Mutex
    },
    thread::spawn,
};

use palette::{Clamp, LinSrgb};
use shark::{
    point::Point,
    shader::{FragThree, Shader},
};
use smart_leds::{RGB8, SmartLedsWrite};

use crate::shaders::to_linsrgb;

struct RenderCtx {
    shader: Arc<dyn Shader<FragThree, Output = LinSrgb<f64>>>,
    points_indexed: Vec<(usize, Point)>,
    time: f64,
}

pub struct Renderer {
    spi_barrier: Arc<Barrier>,
    colors: Arc<Mutex<Vec<RGB8>>>,

    render_workers_barrier: Arc<Barrier>,

    render_workers_ctx_senders: Vec<Sender<RenderCtx>>,

    worker_output_colors_receiver: Receiver<Vec<(usize, RGB8)>>,
}
impl Renderer {
    pub fn new<S: SmartLedsWrite<Color = RGB8> + Send + 'static>(
        num_workers: usize,
        mut strip: S,
    ) -> Self
    where
        S::Error: std::fmt::Debug,
    {
        let spi_barrier = Arc::new(Barrier::new(2));
        let colors = Arc::new(Mutex::new(vec![]));

        let render_workers_barrier = Arc::new(Barrier::new(num_workers + 1));
        let (output_sender, worker_output_colors_receiver) = std::sync::mpsc::channel();

        let mut render_workers_ctx_senders = Vec::new();

        // Spawn render workers
        for _ in 0..num_workers {
            let (ctx_sender, ctx_receiver) = std::sync::mpsc::channel();
            render_workers_ctx_senders.push(ctx_sender);

            let worker_barrier = render_workers_barrier.clone();
            let output_sender = output_sender.clone();
            spawn(move || {
                loop {
                    let ctx: RenderCtx = ctx_receiver.recv().unwrap();

                    let colors = ctx
                        .points_indexed
                        .iter()
                        .map(|(i, point)| {
                            (
                                i,
                                ctx.shader.shade(FragThree {
                                    pos: [point.x, point.y, point.z],
                                    time: ctx.time,
                                }),
                            )
                        })
                        .map(|(i, c)| (i, c.clamp()))
                        .map(|(i, c)| {
                            (
                                *i,
                                RGB8::new(
                                    (c.red * 256.0) as u8,
                                    (c.green * 256.0) as u8,
                                    (c.blue * 256.0) as u8,
                                ),
                            )
                        })
                        .collect();

                    output_sender.send(colors).unwrap();
                    worker_barrier.wait();
                }
            });
        }

        // Spawn SPI writer
        spawn({
            let barrier = spi_barrier.clone();
            let colors = colors.clone();
            move || {
                loop {
                    barrier.wait();
                    strip.write(colors.lock().unwrap().iter().cloned()).unwrap();
                }
            }
        });

        Self {
            spi_barrier,
            colors,

            render_workers_barrier,
            render_workers_ctx_senders,

            worker_output_colors_receiver,
        }
    }

    pub fn render(&self, shader: impl Shader<FragThree> + 'static, points: Vec<Point>, time: f64) {
        let num_workers = self.render_workers_ctx_senders.len();
        let shader = Arc::new(to_linsrgb(shader));

        let points_indexed = points.into_iter().enumerate().collect::<Vec<_>>();
        for (chunk, sender) in points_indexed
            .chunks(points_indexed.len() / num_workers)
            .zip(&self.render_workers_ctx_senders)
        {
            let points_indexed = chunk.to_vec();

            let ctx = RenderCtx {
                shader: shader.clone(),
                points_indexed: points_indexed.clone(),
                time,
            };

            sender.send(ctx).unwrap();
        }
        self.render_workers_barrier.wait();

        let mut new_colors = vec![RGB8::default(); points_indexed.len()];
        for _ in 0..num_workers {
            let colors = self.worker_output_colors_receiver.recv().unwrap();
            for (i, c) in colors {
                new_colors[i] = c;
            }
        }
        // dbg!(&new_colors);
        *self.colors.lock().unwrap() = new_colors;

        self.spi_barrier.wait();
    }
}
