use itertools::Itertools;
use num_complex::Complex;
use rustfft::FftPlanner;
use wasm_bindgen::{prelude::*, Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, *};

const W: u32 = 1 << 12;

// Most tutorials I've seen muck around with once_cell or lazy_static here.
// I'll just spaghetti it out.
fn leak<T>(t: T) -> &'static mut T {
    Box::leak(Box::new(t))
}

#[wasm_bindgen(start)]
pub fn main() {
    let window = leak(window().unwrap());

    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .expect_throw("get canvas");
    let canvas: HtmlCanvasElement = canvas
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    canvas.set_width(W);
    let drawing_ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    let drawing_ctx = leak(drawing_ctx);

    let _ = window
        .navigator()
        .media_devices()
        .expect_throw("media devices")
        .get_user_media_with_constraints(MediaStreamConstraints::new().audio(&true.into()))
        .expect_throw("get audio device")
        .then(leak(Closure::wrap(Box::new(|stream: JsValue| {
            let audio_ctx = AudioContext::new().expect_throw("audio context");
            let source = audio_ctx
                .create_media_stream_source(&stream.into())
                .expect_throw("create stream source");
            let processor = audio_ctx
                .create_script_processor_with_buffer_size(1 << 14)
                .expect_throw("crate script processor");
            processor.set_onaudioprocess(
                leak(Closure::<dyn FnMut(_)>::wrap(Box::new(
                    |sample: JsValue| {
                        let sample: AudioProcessingEvent = sample.into();
                        let sample = sample
                            .input_buffer()
                            .expect_throw("sample input buffer")
                            .get_channel_data(0)
                            .expect_throw("sample channel 0 data");
                        let line = ImageData::new_with_u8_clamped_array(
                            Clamped(&marble(&sample)),
                            sample.len() as u32,
                        )
                        .expect_throw("DCT line data to image");
                        let canvas = drawing_ctx.canvas().unwrap();
                        let window_height = window.inner_height().unwrap().as_f64().unwrap() as u32;
                        if window_height != canvas.height() {
                            let preserve = drawing_ctx.get_image_data(
                                0.,
                                0.,
                                canvas.width() as f64,
                                canvas.height() as f64,
                            );
                            canvas.set_height(window_height);
                            if let Ok(preserve) = preserve {
                                drawing_ctx.put_image_data(&preserve, 0., 0.).ok();
                            }
                        }
                        drawing_ctx
                            .draw_image_with_html_canvas_element(&canvas, 0.0, 1.0)
                            .expect_throw("move view");
                        drawing_ctx
                            .put_image_data(&line, 0., 0.)
                            .expect_throw("draw DCT line");
                    },
                )))
                .as_ref()
                .dyn_ref(),
            );
            source
                .connect_with_audio_node(&processor)
                .expect_throw("connect audio processor");
        }) as Box<_>)));
}

// Proprietary meat smoking code
fn marble(sample: &[f32]) -> Vec<u8> {
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(sample.len());
    let mut sample = sample
        .into_iter()
        .map(|&s| Complex::new(s, 0.))
        .collect::<Vec<_>>();
    fft.process(&mut sample);
    let mut line = vec![0; sample.len() * 4];
    for (&s, (r, g, b, a)) in sample.iter().zip(line.iter_mut().tuples()) {
        let i = ((s.norm_sqr() / 5. + 1.).log2() / 3. - (2. / 3.)).clamp(0., 2.);
        let speck = (149., 31., 38.);
        let schwarte = (234., 200., 186.);
        let schwarz = (0., 0., 0.);
        let (c0, c1, i) = if i < 1. {
            (schwarz, schwarte, i)
        } else {
            (schwarte, speck, i - 1.)
        };
        let ni = 1. - i;
        *a = 255;
        *r = (c0.0 * ni + c1.0 * i) as u8;
        *g = (c0.1 * ni + c1.1 * i) as u8;
        *b = (c0.2 * ni + c1.2 * i) as u8;
    }
    line
}
