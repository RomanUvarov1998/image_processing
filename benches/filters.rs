use criterion::{criterion_group, criterion_main, Criterion};
use image_processing::{img::{*, filter::{filter_option::*, filter_trait::Filter}}, processing::{HaltMessage, ProgressProvider}};
use image_processing::img::filter::linear::LinearCustom;
use image_processing::processing::TaskMsg;
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

	let coeffs: Vec<f64> = vec![
		1.0, 2.0, 1.0,
		0.0, 0.0, 0.0,
		-1.0, -2.0, -1.0,
	];
	let filter = LinearCustom::with_coeffs(coeffs, 
		3, 3, 
		ExtendValue::Closest, 
		NormalizeOption::Normalized);

	let img = create_img(1000, 1000);

	
    let mut group = c.benchmark_group("run filter 100 times");
	group.sample_size(10);
	group.bench_function("filter linear custom img 1000x1000", move |b| {
		b.iter(|| {
			let (tx_prog, _rx_prog) = std::sync::mpsc::channel::<TaskMsg>();
			let (_tx_halt, rx_halt) = std::sync::mpsc::channel::<HaltMessage>();
			let actions_count = filter.get_steps_num(&img);

			let mut prog_prov = ProgressProvider::new(&tx_prog, &rx_halt, actions_count);

			let _res = filter.process(&img, &mut prog_prov);
			prog_prov.assert_all_actions_completed();
		});
	});
	group.finish();
}

criterion_group!(benches, filter_linear_custom);
criterion_main!(benches);