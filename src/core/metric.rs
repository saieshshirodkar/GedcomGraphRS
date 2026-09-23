pub trait Metric {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn center_rel_x(&self) -> f32;
    fn center_rel_y(&self) -> f32;
    fn center_x(&self) -> f32 {
        self.x() + self.center_rel_x()
    }
    fn center_y(&self) -> f32 {
        self.y() + self.center_rel_y()
    }
    fn set_x(&mut self, x: f32);
    fn set_y(&mut self, y: f32);
    fn right(&self) -> f32 {
        self.x() + self.width()
    }
    fn bottom(&self) -> f32 {
        self.y() + self.height()
    }
    fn is_empty_size(&self) -> bool {
        self.width() <= 0.0 && self.height() <= 0.0
    }
    fn has_finite_pos(&self) -> bool {
        self.x().is_finite() && self.y().is_finite()
    }
    fn has_finite_size(&self) -> bool {
        self.width().is_finite() && self.height().is_finite()
    }
    fn has_finite_rect(&self) -> bool {
        self.has_finite_pos() && self.has_finite_size()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn of(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }

    pub fn union(&self, other: &Rect) -> Rect {
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.w).max(other.x + other.w);
        let y2 = (self.y + self.h).max(other.y + other.h);
        Rect {
            x: x1,
            y: y1,
            w: x2 - x1,
            h: y2 - y1,
        }
    }

    pub fn shifted(&self, dx: f32, dy: f32) -> Rect {
        Rect {
            x: self.x + dx,
            y: self.y + dy,
            w: self.w,
            h: self.h,
        }
    }

    pub fn area(&self) -> f32 {
        if self.w <= 0.0 || self.h <= 0.0 {
            0.0
        } else {
            self.w * self.h
        }
    }
}

impl Metric for Rect {
    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }
    fn width(&self) -> f32 {
        self.w
    }
    fn height(&self) -> f32 {
        self.h
    }
    fn center_rel_x(&self) -> f32 {
        self.w / 2.0
    }
    fn center_rel_y(&self) -> f32 {
        self.h / 2.0
    }
    fn set_x(&mut self, x: f32) {
        self.x = x;
    }
    fn set_y(&mut self, y: f32) {
        self.y = y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_math() {
        let r = Rect::of(10.0, 20.0, 30.0, 40.0);
        assert_eq!(r.center_x(), 25.0);
        assert_eq!(r.center_y(), 40.0);
        assert_eq!(r.right(), 40.0);
        assert_eq!(r.bottom(), 60.0);
    }

    #[test]
    fn union_rect() {
        let a = Rect::of(0.0, 0.0, 10.0, 10.0);
        let b = Rect::of(5.0, 5.0, 10.0, 10.0);
        let u = a.union(&b);
        assert_eq!((u.x, u.y, u.w, u.h), (0.0, 0.0, 15.0, 15.0));
    }

    #[test]
    fn shift_and_area() {
        let a = Rect::of(1.0, 2.0, 3.0, 4.0);
        let s = a.shifted(1.0, -1.0);
        assert_eq!((s.x, s.y), (2.0, 1.0));
        assert_eq!(a.area(), 12.0);
        assert_eq!(Rect::of(0.0, 0.0, 0.0, 5.0).area(), 0.0);
    }

    #[test]
    fn finite_checks() {
        let a = Rect::of(1.0, 2.0, 3.0, 4.0);
        assert!(a.has_finite_rect());
        assert!(!a.is_empty_size());
        assert!(Rect::default().is_empty_size());
    }

    #[test]
    fn setters() {
        let mut a = Rect::default();
        a.set_x(7.0);
        a.set_y(8.0);
        assert_eq!((a.x(), a.y()), (7.0, 8.0));
    }
}
