use itertools::Itertools;
use rustdct::DctPlanner;
use wasm_bindgen::{prelude::*, Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, *};

fn leak<T>(t: T) -> &'static mut T {
    Box::leak(Box::new(t))
}

const W: u32 = 1 << 12;

#[wasm_bindgen]
pub fn run() {
    let window = window().unwrap();

    let document = window.document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .expect_throw("get canvas");
    let canvas: HtmlCanvasElement = canvas
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    canvas.set_height(window.inner_height().unwrap().as_f64().unwrap() as u32);
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
                        let mut sample = sample
                            .input_buffer()
                            .expect_throw("sample input buffer")
                            .get_channel_data(0)
                            .expect_throw("sample channel 0 data");
                        let mut planner = DctPlanner::new();
                        let dct = planner.plan_dct2(sample.len());
                        dct.process_dct2(&mut sample);
                        let mut line = vec![0; sample.len() * 4];
                        for (&s, (r, g, b, a)) in sample.iter().zip(line.iter_mut().tuples()) {
                            *a = 255;
                            let s = s * s * 127.;
                            *r = (s * 0.427).min(255.) as u8;
                            *g = (s * 0.122).min(255.) as u8;
                            *b = (s * 0.149).min(255.) as u8;
                        }
                        let line = ImageData::new_with_u8_clamped_array(
                            Clamped(&line),
                            sample.len() as u32,
                        )
                        .expect_throw("DCT line data to image");
                        drawing_ctx
                            .draw_image_with_html_canvas_element(
                                &drawing_ctx.canvas().unwrap(),
                                0.0,
                                1.0,
                            )
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
