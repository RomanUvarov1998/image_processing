// use crate::{filter::{color_channel::Rgb2Gray, non_linear::MedianFilter}, processing::{FilterBase, TaskMsg}};

#[test]
fn test1() {
	// let (tx, rx) = std::sync::mpsc::channel::<TaskMsg>();
	// let bw = super::BackgroundWorker::new(tx);

	// bw.unlocked().add_step(Box::new(MedianFilter::default()) as FilterBase);
	// bw.unlocked().add_step(Box::new(Rgb2Gray::default()) as FilterBase);

	// assert_eq!(bw.unlocked().get_steps_count(), 2);

	// let path: &str = r"C:\Users\Роман\Documents\__Виллевальде\Курсач\bmps\3.bmp";
	// let result = bw.unlocked().try_load_initial_img(path);
	// if result.is_err() {
	// 	println!("For path '{}': {:?}", path, result);
	// 	panic!();
	// }
	
	// for i in 0..2 {
	// 	bw.unlocked().start_processing(i, None);

	// 	// wait for completed msg
	// 	loop {
	// 		if let TaskMsg::Finished = rx.recv().unwrap() {
	// 			break;
	// 		}
	// 	}

	// 	bw.unlocked().get_task_result().unwrap();
	// }
}