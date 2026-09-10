use gtk::{glib, prelude::*};

glib::wrapper! {
    pub struct MasonryLayout(ObjectSubclass<imp::MasonryLayout>)
        @extends gtk::LayoutManager;
}

impl Default for MasonryLayout {
    fn default() -> Self {
        glib::Object::new()
    }
}

mod imp {
    use super::*;
    use gtk::subclass::prelude::*;

    #[derive(Default)]
    pub struct MasonryLayout;

    #[glib::object_subclass]
    impl ObjectSubclass for MasonryLayout {
        const NAME: &'static str = "MasonryLayout";
        type Type = super::MasonryLayout;
        type ParentType = gtk::LayoutManager;
    }

    impl ObjectImpl for MasonryLayout {}

    impl LayoutManagerImpl for MasonryLayout {
        fn request_mode(&self, _widget: &gtk::Widget) -> gtk::SizeRequestMode {
            gtk::SizeRequestMode::HeightForWidth
        }

        // returns minimum, natural, minimum_baseline, natural_baseline
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
                let height = layout(widget, width, |_, _| {});
                (height, height, -1, -1)
            }
        }

        fn allocate(&self, widget: &gtk::Widget, width: i32, _height: i32, _baseline: i32) {
            layout(widget, width, |child, allocation| {
                child.size_allocate(allocation, -1);
            });
        }
    }
}

const SPACING: i32 = 10;

fn children(widget: &gtk::Widget) -> impl Iterator<Item = gtk::Widget> {
    std::iter::successors(widget.first_child(), |child| child.next_sibling())
        .filter(|child| child.should_layout())
}

fn measure_width(widget: &gtk::Widget) -> (i32, i32) {
    let mut minimum = 0;
    let mut natural = 0;
    let mut count = 0;
    for child in children(widget) {
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

// Measurement and allocation use the same packing so the scroller gets the full height.
fn layout(
    widget: &gtk::Widget,
    width: i32,
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
    for child in children(widget) {
        let column = usize::from(columns == 2 && heights[1] < heights[0]);
        let x = column as i32 * (column_width + SPACING);
        let child_width = if column == columns - 1 {
            width - x
        } else {
            column_width
        };
        let (_, height, _, _) = child.measure(gtk::Orientation::Vertical, child_width);
        let y = heights[column];
        let x = if widget.direction() == gtk::TextDirection::Rtl {
            width - x - child_width
        } else {
            x
        };
        place(&child, &gtk::Allocation::new(x, y, child_width, height));
        heights[column] = y + height + SPACING;
    }
    (heights[0].max(heights[1]) - SPACING).max(0)
}
