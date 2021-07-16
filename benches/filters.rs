use criterion::{criterion_group, criterion_main, Criterion};
use image_processing::{img::{*, filter::{FilterBase, filter_option::*}}};
use image_processing::img::filter::linear::LinearCustom;
use image_processing::processing::{TaskMsg, BackgroundWorker, ProcTask};
use fltk::enums::ColorDepth;

fn create_layer(w: usize, h: usize, channel: ImgChannel) -> ImgLayer {
	let mat = Matrix2D::empty_with_size(w, h);
	ImgLayer::new(mat, channel)
}

fn create_img(w: usize, h: usize) -> Img {
	let layers: Vec<ImgLayer> = vec![
		create_layer(w, h, ImgChannel::A),
		create_layer(w, h, ImgChannel::R),
		create_layer(w, h, ImgChannel::G),
		create_layer(w, h, ImgChannel::B),
	];
	Img::new(w, h, layers, ColorDepth::Rgba8)
}

pub fn filter_linear_custom(c: &mut Criterion) {
	let (tx_prog, rx_prog) = std::sync::mpsc::channel::<TaskMsg>();
	let mut bw = BackgroundWorker::new(tx_prog);

	let img = create_img(1000, 1000);
	bw.locked().set_initial_img(img);
	// let path = r"C:\Users\Роман\Documents\__Виллевальде\Курсач\1.jpg";
	// bw.start_task(ImportTask::new(path.to_string()));
	// loop {
	// 	if let TaskMsg::Finished = rx_prog.recv().unwrap() {
	// 		break;
	// 	}
	// }

	let coeffs: Vec<f64> = vec![
		1.0, 2.0, 1.0,
		0.0, 0.0, 0.0,
		-1.0, -2.0, -1.0,
	];
	let filter = LinearCustom::with_coeffs(coeffs, 
		3, 3, 
		ExtendValue::Closest, 
		NormalizeOption::Normalized);
	bw.locked().add_step(Box::new(filter) as FilterBase);

	c.bench_function("filter linear custom", move |b| {
		b.iter(|| {
			bw.start_task(ProcTask::new(0, None));
			loop {
				if let TaskMsg::Finished = rx_prog.recv().unwrap() {
					break;
				}
			}
		});
	});
}

criterion_group!(benches, filter_linear_custom);
criterion_main!(benches);