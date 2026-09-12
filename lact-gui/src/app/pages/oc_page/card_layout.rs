use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::OnceCell;

#[derive(Clone, Copy, Debug)]
pub enum ColumnBias {
    Left,
    Right,
}

glib::wrapper! {
    pub struct CardLayout(ObjectSubclass<imp::CardLayout>)
        @extends gtk::LayoutManager;
}

impl CardLayout {
    pub fn new(column_bias: impl Into<Vec<ColumnBias>>) -> Self {
        let layout: Self = glib::Object::new();
        layout.imp().column_bias.set(column_bias.into()).unwrap();
        layout
    }
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct CardLayout {
        pub column_bias: OnceCell<Vec<ColumnBias>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CardLayout {
        const NAME: &'static str = "CardLayout";
        type Type = super::CardLayout;
        type ParentType = gtk::LayoutManager;
    }

    impl ObjectImpl for CardLayout {}

    impl LayoutManagerImpl for CardLayout {
        fn request_mode(&self, _widget: &gtk::Widget) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }

        fn measure(
            &self,
            widget: &gtk::Widget,
            orientation: gtk::Orientation,
            for_size: i32,
        ) -> (i32, i32, i32, i32) {
            let (minimum, natural) = measure_width(widget);
            if orientation == gtk::Orientation::Horizontal {
                (minimum, natural, -1, -1)
            } else {
                let width = if for_size < 0 { natural } else { for_size };
                let height = layout(widget, width, self.column_bias.get().unwrap(), |_, _| {});
                (height, height, -1, -1)
            }
        }

        fn allocate(&self, widget: &gtk::Widget, width: i32, _height: i32, _baseline: i32) {
            layout(
                widget,
                width,
                self.column_bias.get().unwrap(),
                |child, allocation| {
                    child.size_allocate(allocation, -1);
                },
            );
        }
    }
}

const SPACING: i32 = 15;

fn children(widget: &gtk::Widget) -> impl Iterator<Item = (usize, gtk::Widget)> {
    std::iter::successors(widget.first_child(), |child| child.next_sibling())
        .enumerate()
        .filter(|(_, child)| child.should_layout())
}

fn measure_width(widget: &gtk::Widget) -> (i32, i32) {
    let mut minimum = 0;
    let mut natural = 0;
    let mut count = 0;
    for (_, child) in children(widget) {
        let (child_minimum, child_natural, _, _) = child.measure(gtk::Orientation::Horizontal, -1);
        minimum = minimum.max(child_minimum);
        natural = natural.max(child_natural);
        count += 1;
    }
    if count > 1 {
        natural = natural * 2 + SPACING;
    }
    (minimum, natural)
}

// Measurement and allocation share placement so the scroller gets the full height.
fn layout(
    widget: &gtk::Widget,
    width: i32,
    column_bias: &[ColumnBias],
    mut place: impl FnMut(&gtk::Widget, &gtk::Allocation),
) -> i32 {
    let (minimum, _) = measure_width(widget);
    let columns = if children(widget).count() > 1 && width >= minimum * 2 + SPACING {
        2
    } else {
        1
    };
    let column_width = ((width - SPACING * (columns as i32 - 1)) / columns as i32).max(0);
    let mut heights = [0, 0];
    for (index, child) in children(widget) {
        let column = usize::from(columns == 2 && matches!(column_bias[index], ColumnBias::Right));
        let x = column as i32 * (column_width + SPACING);
        let child_width = if column == columns - 1 {
            width - x
        } else {
            column_width
        };
        let (_, height, _, _) = child.measure(gtk::Orientation::Vertical, child_width);
        let y = heights[column];
        place(&child, &gtk::Allocation::new(x, y, child_width, height));
        heights[column] = y + height + SPACING;
    }
    (heights[0].max(heights[1]) - SPACING).max(0)
}
