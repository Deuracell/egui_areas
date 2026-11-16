//! Rounded area corners. egui cannot clip to a rounded shape, so the corners
//! are cut by painting the backdrop colour over the square corner wedges.

pub(super) const CORNER_RADIUS: f32 = 6.0;
const ARC_SEGMENTS: usize = 8;

/// The colour seen between and behind areas.
pub(super) fn backdrop(ui: &egui::Ui) -> egui::Color32 {
    ui.visuals().extreme_bg_color
}

pub(super) fn radius() -> egui::CornerRadius {
    egui::CornerRadius::same(CORNER_RADIUS as u8)
}

/// Rounded on top only, for the header strip.
pub(super) fn top_radius() -> egui::CornerRadius {
    egui::CornerRadius { nw: CORNER_RADIUS as u8, ne: CORNER_RADIUS as u8, sw: 0, se: 0 }
}

/// Paints the four outside-corner wedges of `rect` in `color`.
pub(super) fn mask_corners(painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
    let r = CORNER_RADIUS;
    // (corner point, direction into the rect)
    let corners = [
        (rect.left_top(), egui::vec2(1.0, 1.0)),
        (rect.right_top(), egui::vec2(-1.0, 1.0)),
        (rect.left_bottom(), egui::vec2(1.0, -1.0)),
        (rect.right_bottom(), egui::vec2(-1.0, -1.0)),
    ];
    for (p, d) in corners {
        let centre = p + d * r;
        let mut mesh = egui::Mesh::default();
        mesh.colored_vertex(p, color);
        for i in 0..=ARC_SEGMENTS {
            let t = i as f32 / ARC_SEGMENTS as f32 * std::f32::consts::FRAC_PI_2;
            mesh.colored_vertex(centre + egui::vec2(-d.x * r * t.cos(), -d.y * r * t.sin()), color);
        }
        for i in 0..ARC_SEGMENTS as u32 {
            mesh.add_triangle(0, i + 1, i + 2);
        }
        painter.add(egui::Shape::mesh(mesh));
    }
}
