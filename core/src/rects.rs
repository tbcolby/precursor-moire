//! Bitmap to rectangles. Each horizontal run of set pixels is merged with the
//! identical run in the row above, so vertical bands and rings that repeat
//! down the screen become a few tall rectangles instead of one GAM object per
//! pixel row (the old display path drew up to 15,436 runs per frame).

/// Inclusive rectangle `(x0, y0, x1, y1)`.
pub type Rect = (i32, i32, i32, i32);

/// Bytes per bitmap row.
pub fn row_bytes(w: i32) -> usize { (w.max(0) as usize).div_ceil(8) }

pub fn get(bits: &[u8], w: i32, x: i32, y: i32) -> bool {
    bits[y as usize * row_bytes(w) + x as usize / 8] & (0x80 >> (x as usize % 8)) != 0
}

fn runs_of_row(bits: &[u8], w: i32, y: i32) -> Vec<(i32, i32)> {
    let mut runs = Vec::new();
    let mut x = 0;
    while x < w {
        if get(bits, w, x, y) {
            let start = x;
            while x < w && get(bits, w, x, y) {
                x += 1;
            }
            runs.push((start, x - 1));
        } else {
            x += 1;
        }
    }
    runs
}

/// Cover exactly the set pixels of a `w` x `h` bitmap with disjoint rectangles.
pub fn rects(bits: &[u8], w: i32, h: i32) -> Vec<Rect> {
    let mut out = Vec::new();
    // open runs from the previous row: (x0, x1, first row)
    let mut open: Vec<(i32, i32, i32)> = Vec::new();
    for y in 0..h {
        let runs = runs_of_row(bits, w, y);
        let mut next = Vec::with_capacity(runs.len());
        let mut used = vec![false; open.len()];
        let mut j = 0;
        for &(x0, x1) in &runs {
            while j < open.len() && open[j].0 < x0 {
                j += 1;
            }
            if j < open.len() && open[j].0 == x0 && open[j].1 == x1 {
                used[j] = true;
                next.push((x0, x1, open[j].2));
            } else {
                next.push((x0, x1, y));
            }
        }
        for (k, o) in open.iter().enumerate() {
            if !used[k] {
                out.push((o.0, o.2, o.1, y - 1));
            }
        }
        open = next;
    }
    for o in open {
        out.push((o.0, o.2, o.1, h - 1));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bitmap(w: i32, h: i32, f: impl Fn(i32, i32) -> bool) -> Vec<u8> {
        let mut b = vec![0u8; row_bytes(w) * h as usize];
        for y in 0..h {
            for x in 0..w {
                if f(x, y) {
                    b[y as usize * row_bytes(w) + x as usize / 8] |= 0x80 >> (x % 8);
                }
            }
        }
        b
    }

    fn check_cover(b: &[u8], w: i32, h: i32) -> usize {
        let rs = rects(b, w, h);
        let mut seen = vec![0u8; (w * h) as usize];
        for &(x0, y0, x1, y1) in &rs {
            assert!(x0 <= x1 && y0 <= y1 && x0 >= 0 && y0 >= 0 && x1 < w && y1 < h);
            for y in y0..=y1 {
                for x in x0..=x1 {
                    seen[(y * w + x) as usize] += 1;
                }
            }
        }
        for y in 0..h {
            for x in 0..w {
                assert_eq!(seen[(y * w + x) as usize], get(b, w, x, y) as u8, "pixel {x},{y}");
            }
        }
        rs.len()
    }

    #[test]
    fn covers_exactly_the_set_pixels_without_overlap() {
        let (w, h) = (45, 31); // not a multiple of 8
        check_cover(&bitmap(w, h, |x, y| (x * 7 + y * 3) % 5 < 2), w, h);
        check_cover(&bitmap(w, h, |x, y| (x - 20) * (x - 20) + (y - 15) * (y - 15) < 100), w, h);
        check_cover(&bitmap(w, h, |_, _| true), w, h);
        check_cover(&bitmap(w, h, |_, _| false), w, h);
    }

    #[test]
    fn vertical_bands_merge_into_one_rectangle_each() {
        let (w, h) = (120, 100);
        let n = check_cover(&bitmap(w, h, |x, _| x % 10 < 5), w, h);
        assert_eq!(n, 12);
    }

    #[test]
    fn full_and_single_pixel_cases() {
        let n = check_cover(&bitmap(16, 16, |_, _| true), 16, 16);
        assert_eq!(n, 1);
        let n = check_cover(&bitmap(16, 16, |x, y| x == 3 && y == 4), 16, 16);
        assert_eq!(n, 1);
    }
}
