use criterion::{criterion_group, criterion_main, Criterion};
use image_processing::{img::{*, filter::{filter_option::*, filter_trait}}, processing::task_info_channel};
use image_processing::img::filter::{linear::*, non_linear::*, color_channel::*};
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

fn run_filter<F: filter_trait::Filter>(img: &Img, filter: F) {
	let (executor_handle, _delegator_handle) = task_info_channel();
	executor_handle.reset(filter.get_steps_num(img));
	let _res = filter.process(&img, &executor_handle);
	executor_handle.assert_all_actions_completed();
}

pub fn filter_linear_custom(c: &mut Criterion) {
	let img = create_img(1000, 1000);
	
    let mut group = c.benchmark_group("run filter 10 times");
	group.sample_size(100);

	group.bench_function("filter linear custom img 1000x1000", |b| {
		b.iter(|| run_filter(&img, LinearCustom::default()));
	});

	group.bench_function("filter linear gaussian img 1000x1000", |b| {
		b.iter(|| run_filter(&img, LinearGaussian::default()));
	});

	group.bench_function("filter linear mean img 1000x1000", |b| {
		b.iter(|| run_filter(&img, LinearMean::default()));
	});

	group.bench_function("filter CannyEdgeDetection img 1000x1000", |b| {
		b.iter(|| run_filter(&img, CannyEdgeDetection::default()));
	});

	group.bench_function("filter HistogramLocalContrast img 1000x1000", |b| {
		b.iter(|| run_filter(&img, HistogramLocalContrast::default()));
	});

	group.bench_function("filter MedianFilter img 1000x1000", |b| {
		b.iter(|| run_filter(&img, MedianFilter::default()));
	});

	group.bench_function("filter CutBrightness img 1000x1000", |b| {
		b.iter(|| run_filter(&img, CutBrightness::default()));
	});

	group.bench_function("filter EqualizeHist img 1000x1000", |b| {
		b.iter(|| run_filter(&img, EqualizeHist::default()));
	});

	group.bench_function("filter ExtractChannel img 1000x1000", |b| {
		b.iter(|| run_filter(&img, ExtractChannel::default()));
	});

	group.bench_function("filter NeutralizeChannel img 1000x1000", |b| {
		b.iter(|| run_filter(&img, NeutralizeChannel::default()));
	});

	group.bench_function("filter Rgb2Gray img 1000x1000", |b| {
		b.iter(|| run_filter(&img, Rgb2Gray::default()));
	});

	group.finish();
}

criterion_group!(benches, filter_linear_custom);
criterion_main!(benches);