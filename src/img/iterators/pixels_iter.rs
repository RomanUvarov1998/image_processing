use super::{PixelPos, PixelsArea};
use crate::processing::ExecutorHandle;

pub trait PixelsAreaIter<'area>: Iterator<Item = PixelPos> {
    fn area(&self) -> &'area PixelsArea;
}

pub struct PixelsIter<'area> {
    area: &'area PixelsArea,
    cur_pos: PixelPos,
}

impl<'area> PixelsIter<'area> {
    pub fn for_area(area: &'area PixelsArea) -> Self {
        PixelsIter {
            area,
            cur_pos: area.top_left(),
        }
    }

    pub fn track_progress(self, executor_handle: &'area mut ExecutorHandle) -> PixelsProgressIter {
        PixelsProgressIter::new(self, executor_handle)
    }
}

impl<'area> PixelsAreaIter<'area> for PixelsIter<'area> {
    fn area(&self) -> &'area PixelsArea {
        self.area
    }
}

impl<'area> Iterator for PixelsIter<'area> {
    type Item = PixelPos;

    fn next(&mut self) -> Option<PixelPos> {
        let curr = self.cur_pos;

        self.cur_pos.col += 1;

        if self.cur_pos.col > self.area().bottom_right().col {
            self.cur_pos.col = self.area().top_left().col;
            self.cur_pos.row += 1;
        }

        return if self.area().contains(curr) {
            Some(curr)
        } else {
            None
        };
    }
}

pub struct PixelsProgressIter<'handle_and_area> {
    iter: PixelsIter<'handle_and_area>,
    executor_handle: &'handle_and_area mut ExecutorHandle,
    cur_row_num: usize,
}

impl<'handle_and_area> PixelsProgressIter<'handle_and_area> {
    fn new(
        iter: PixelsIter<'handle_and_area>,
        executor_handle: &'handle_and_area mut ExecutorHandle,
    ) -> Self {
        let cur_row_num = iter.area().top_left().row;
        PixelsProgressIter {
            iter,
            executor_handle,
            cur_row_num,
        }
    }
}

impl<'handle_and_area> PixelsAreaIter<'handle_and_area> for PixelsProgressIter<'handle_and_area> {
    fn area(&self) -> &'handle_and_area PixelsArea {
        self.iter.area()
    }
}

impl Iterator for PixelsProgressIter<'_> {
    type Item = PixelPos;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(p) => {
                if p.row > self.cur_row_num {
                    self.cur_row_num = p.row;
                    self.executor_handle.complete_action().ok()?;
                }

                Some(p)
            }
            None => {
                self.executor_handle.complete_action().ok()?;
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::processing::{DelegatorHandle, TaskState, TaskStop};

    use super::PixelsIter;
    use crate::{
        img::{PixelPos, PixelsArea, PixelsAreaIter},
        processing::create_task_info_channel,
    };

    #[test]
    fn for_area() {
        let top_left = PixelPos::new(1, 2);
        let bottom_right = PixelPos::new(3, 5);
        let area = PixelsArea::new(top_left, bottom_right);

        let iter = PixelsIter::for_area(&area);
        assert_eq!(iter.area(), &area);
        assert_eq!(iter.cur_pos, area.top_left());
    }

    #[test]
    fn progress_iter() {
        let area = PixelsArea::new(PixelPos::new(1, 2), PixelPos::new(3, 4));

        let (mut ex, del) = create_task_info_channel();

        ex.reset(3);

        let mut iter = area.iter_pixels().track_progress(&mut ex);

        let check_percents = |d: &DelegatorHandle, p: usize| {
            if let TaskState::InProgress { percents } = d.get_task_state() {
                assert_eq!(percents, p);
            } else {
                panic!("State is not 'InProgress': {:?}", d.get_task_state());
            }
        };

        check_percents(&del, 0);
        assert_eq!(iter.next(), Some(PixelPos::new(1, 2)));
        check_percents(&del, 100 * 0 / 3);
        assert_eq!(iter.next(), Some(PixelPos::new(1, 3)));
        check_percents(&del, 100 * 0 / 3);
        assert_eq!(iter.next(), Some(PixelPos::new(1, 4)));
        check_percents(&del, 100 * 0 / 3);

        assert_eq!(iter.next(), Some(PixelPos::new(2, 2)));
        check_percents(&del, 100 * 1 / 3);
        assert_eq!(iter.next(), Some(PixelPos::new(2, 3)));
        check_percents(&del, 100 * 1 / 3);
        assert_eq!(iter.next(), Some(PixelPos::new(2, 4)));
        check_percents(&del, 100 * 1 / 3);

        assert_eq!(iter.next(), Some(PixelPos::new(3, 2)));
        check_percents(&del, 100 * 2 / 3);
        assert_eq!(iter.next(), Some(PixelPos::new(3, 3)));
        check_percents(&del, 100 * 2 / 3);
        assert_eq!(iter.next(), Some(PixelPos::new(3, 4)));
        check_percents(&del, 100 * 2 / 3);

        assert_eq!(iter.next(), None);
        check_percents(&del, 100 * 3 / 3);
    }

    #[test]
    fn progress_iter_halt() {
        let area = PixelsArea::new(PixelPos::new(1, 2), PixelPos::new(3, 4));

        let (mut ex, del) = create_task_info_channel();

        ex.reset(3);

        let mut iter = area.iter_pixels().track_progress(&mut ex);

        let check_percents = |d: &DelegatorHandle, p: usize| {
            if let TaskState::InProgress { percents } = d.get_task_state() {
                assert_eq!(percents, p);
            } else {
                panic!("State is not 'InProgress': {:?}", d.get_task_state());
            }
        };

        check_percents(&del, 0);
        assert_eq!(iter.next(), Some(PixelPos::new(1, 2)));
        check_percents(&del, 100 * 0 / 3);
        assert_eq!(iter.next(), Some(PixelPos::new(1, 3)));
        check_percents(&del, 100 * 0 / 3);
        assert_eq!(iter.next(), Some(PixelPos::new(1, 4)));
        check_percents(&del, 100 * 0 / 3);

        assert_eq!(iter.next(), Some(PixelPos::new(2, 2)));
        check_percents(&del, 100 * 1 / 3);
        assert_eq!(iter.next(), Some(PixelPos::new(2, 3)));
        check_percents(&del, 100 * 1 / 3);
        assert_eq!(iter.next(), Some(PixelPos::new(2, 4)));
        check_percents(&del, 100 * 1 / 3);

        del.halt_task();
        assert_eq!(iter.next(), None);
        if let TaskState::Finished { result } = del.get_task_state() {
            if let Err(TaskStop::Halted) = result {
                // ok
            } else {
                panic!("task state is not 'Finished and Halted'");
            }
        } else {
            panic!("task state is not 'Finished'");
        }
    }
}
