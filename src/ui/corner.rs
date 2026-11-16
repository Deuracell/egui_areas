/// One of an area's four corners, where split/join/swap gestures start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    pub const ALL: [Corner; 4] = [Corner::TopLeft, Corner::TopRight, Corner::BottomLeft, Corner::BottomRight];

    /// `+1` if moving right goes into the area from this corner, `-1` if it leaves.
    pub fn inward_x(self) -> f32 {
        match self {
            Corner::TopLeft | Corner::BottomLeft => 1.0,
            Corner::TopRight | Corner::BottomRight => -1.0,
        }
    }

    /// `+1` if moving down goes into the area from this corner, `-1` if it leaves.
    pub fn inward_y(self) -> f32 {
        match self {
            Corner::TopLeft | Corner::TopRight => 1.0,
            Corner::BottomLeft | Corner::BottomRight => -1.0,
        }
    }

    /// The square grab zone at this corner, inside `area`.
    pub fn hotspot(self, area: egui::Rect, size: f32) -> egui::Rect {
        let min = egui::pos2(
            if self.inward_x() > 0.0 { area.min.x } else { area.max.x - size },
            if self.inward_y() > 0.0 { area.min.y } else { area.max.y - size },
        );
        egui::Rect::from_min_size(min, egui::vec2(size, size))
    }
}
