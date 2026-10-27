use super::gesture::Gesture;
use crate::Axis;

/// Paints the ghost preview of a corner gesture on top of the areas.
pub(super) fn paint_gesture(ui: &egui::Ui, bounds: egui::Rect, source: egui::Rect, gesture: &Gesture) {
    let painter = ui.painter();
    let accent = ui.visuals().selection.stroke.color;
    let tint = accent.gamma_multiply(0.25);
    let line = egui::Stroke::new(2.0, accent);

    match *gesture {
        Gesture::Idle => {}
        Gesture::Split { axis, ratio, guide } => {
            let (first, second) = match axis {
                Axis::Horizontal => {
                    let x = source.min.x + source.width() * ratio;
                    (
                        egui::Rect::from_min_max(source.min, egui::pos2(x, source.max.y)),
                        egui::Rect::from_min_max(egui::pos2(x, source.min.y), source.max),
                    )
                }
                Axis::Vertical => {
                    let y = source.min.y + source.height() * ratio;
                    (
                        egui::Rect::from_min_max(source.min, egui::pos2(source.max.x, y)),
                        egui::Rect::from_min_max(egui::pos2(source.min.x, y), source.max),
                    )
                }
            };
            painter.rect_filled(second, 0.0, tint);
            painter.rect_stroke(first, 0.0, line, egui::StrokeKind::Inside);
            painter.rect_stroke(second, 0.0, line, egui::StrokeKind::Inside);
            if let Some(c) = guide {
                match axis {
                    Axis::Horizontal => painter.vline(c, bounds.y_range(), egui::Stroke::new(1.0, accent)),
                    Axis::Vertical => painter.hline(bounds.x_range(), c, egui::Stroke::new(1.0, accent)),
                };
            }
        }
        Gesture::Join { target_rect, result, .. } => {
            // The neighbour being taken over is marked lightly; the area that
            // will cover the stretch, which may be only part of the dragged
            // one, is filled and outlined.
            painter.rect_stroke(target_rect, 0.0, egui::Stroke::new(1.0, accent), egui::StrokeKind::Inside);
            painter.rect_filled(result, 0.0, tint);
            painter.rect_stroke(result, 0.0, line, egui::StrokeKind::Inside);
            painter.arrow(
                source.center(),
                (target_rect.center() - source.center()).normalized() * 40.0,
                egui::Stroke::new(3.0, egui::Color32::WHITE),
            );
        }
        Gesture::PopOut => {
            painter.rect_filled(source, 0.0, tint);
            painter.rect_stroke(source, 0.0, line, egui::StrokeKind::Inside);
            // A smaller outline offset from the area: "this becomes a window".
            let window = source.shrink2(source.size() * 0.2).translate(egui::vec2(12.0, -12.0));
            painter.rect_filled(window, 4.0, ui.visuals().panel_fill);
            painter.rect_stroke(window, 4.0, line, egui::StrokeKind::Inside);
        }
        Gesture::Swap { target_rect, .. } => {
            for r in [source, target_rect] {
                painter.rect_filled(r, 0.0, tint);
                painter.rect_stroke(r, 0.0, line, egui::StrokeKind::Inside);
            }
        }
    }
}
