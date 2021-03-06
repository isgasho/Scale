use cgmath::InnerSpace;
use cgmath::Vector2;

#[derive(Clone, Copy)]
pub struct Ray {
    pub from: Vector2<f32>,
    pub dir: Vector2<f32>,
}

pub fn intersection_point(r1: Ray, r2: Ray) -> Option<Vector2<f32>> {
    let r2dir_nor = Vector2::new(-r2.dir.y, r2.dir.x);
    let r1dir_nor = Vector2::new(-r1.dir.y, r1.dir.x);

    let div = r1.dir.dot(-r2dir_nor);

    let p_diff = r1.from - r2.from;
    let t = r2dir_nor.dot(p_diff);
    let s = r1dir_nor.dot(p_diff);

    if t * div > 0.0 && s * div > 0.0 {
        Some(r1.from + r1.dir * t / div)
    } else {
        None
    }
}

pub fn both_dist_to_inter(r1: Ray, r2: Ray) -> Option<(f32, f32)> {
    let r2dir_nor = Vector2::new(-r2.dir.y, r2.dir.x);
    let r1dir_nor = Vector2::new(-r1.dir.y, r1.dir.x);

    let p_diff = r1.from - r2.from;

    let div = r1.dir.dot(-r2dir_nor);

    let t = r2dir_nor.dot(p_diff);
    let s = r1dir_nor.dot(p_diff);

    if t * div > 0.0 && s * div > 0.0 {
        Some((t / div, s / div))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_inter() {
        let x = Ray {
            from: [0.0, 2.0].into(),
            dir: [1.0, 0.0].into(),
        };

        let y = Ray {
            from: [2.0, 0.0].into(),
            dir: [0.0, 1.0].into(),
        };

        let r = intersection_point(x, y);

        assert!(r.is_some());
        if let Some(v) = r {
            assert_eq!(v.x, 2.0);
            assert_eq!(v.y, 2.0);
        }
    }
}
