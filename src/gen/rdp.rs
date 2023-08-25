use vek::Vec2;

/// Simplify a line string by removing vertices that don't create enough of a bend.
///
/// Source: https://rosettacode.org/wiki/Ramer-Douglas-Peucker_line_simplification#An_implementation_of_the_algorithm
pub fn ramer_douglas_peucker(points: &[Vec2<f64>], epsilon: f64) -> Vec<Vec2<f64>> {
    puffin::profile_scope!("Ramer Douglas Peucker");

    let mut result = Vec::new();

    if !points.is_empty() && epsilon >= 0.0 {
        // Always add the first point
        result.push(points[0]);

        ramer_douglas_peucker_step(points, epsilon, &mut result);
    }

    result
}

/// Recursive implementation of the Ramer Douglas Peucker algorithm.
fn ramer_douglas_peucker_step(points: &[Vec2<f64>], epsilon: f64, result: &mut Vec<Vec2<f64>>) {
    let len = points.len();
    if len < 2 {
        return;
    }

    let mut max_dist = 0.0;
    let mut index = 0;
    for i in 1..len - 1 {
        let dist = perp_dist(points[i], points[0], points[len - 1]);
        if dist > max_dist {
            max_dist = dist;
            index = i;
        }
    }

    if max_dist > epsilon {
        ramer_douglas_peucker_step(&points[0..=index], epsilon, result);
        ramer_douglas_peucker_step(&points[index..len], epsilon, result);
    } else {
        result.push(points[len - 1]);
    }
}

/// Calculate perpendicular distance between a point and a line segment.
fn perp_dist(point: Vec2<f64>, line1: Vec2<f64>, line2: Vec2<f64>) -> f64 {
    let delta = line2 - line1;

    (point.x * delta.y - point.y * delta.x + line2.x * line1.y - line2.y * line1.x).abs()
        / delta.magnitude()
}
