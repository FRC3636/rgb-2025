use shark::point::{Point, primitives::line};
use shrewnit::{Dimension, Inches, Meters, ScalarExt, to};

pub fn test_strip() -> impl Iterator<Item = Point> + Clone {
    line(Point::new(-18.0, 0.0, 0.0), Point::new(18.0, 0.0, 0.0), 72)
}

pub fn box_tube_to_intake() -> impl Iterator<Item = Point> + Clone {
    let segment_length = 17.0f64 * Inches;
    let leds_per_segment = 27;

    line(
        Point {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        Point {
            x: 0.0,
            y: to!(segment_length in Meters),
            z: 0.0,
        },
        leds_per_segment,
    )
    .chain(line(
        Point {
            x: 0.0,
            y: to!(segment_length in Meters),
            z: 0.0,
        },
        Point {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        leds_per_segment,
    ))
    .chain(line(
        Point {
            x: -5.0,
            y: 0.0,
            z: 0.0,
        },
        Point {
            x: -5.0,
            y: to!(segment_length in Meters),
            z: 0.0,
        },
        leds_per_segment,
    ))
    .chain(line(
        Point {
            x: segment_length.to::<Meters>() / 2.0,
            y: 20.0.inches().to::<Meters>(),
            z: 0.0,
        },
        Point {
            x: -(segment_length.to::<Meters>() / 2.0),
            y: 20.0.inches().to::<Meters>(),
            z: 0.0,
        },
        leds_per_segment,
    ))
}

pub fn underglow() -> impl Iterator<Item = Point> + Clone {
    let horizontal_offset = 13.0f64 * Inches;
    let vertical_offset = 14.0f64 * Inches;

    line(
        Point {
            x: -horizontal_offset.to::<Meters>() / 2.0,
            y: 0.0,
            z: vertical_offset.to::<Meters>(),
        },
        Point {
            x: horizontal_offset.to::<Meters>() / 2.0,
            y: 0.0,
            z: vertical_offset.to::<Meters>(),
        },
        23,
    ).chain(
        line(
            Point {
                x: horizontal_offset.to::<Meters>(),
                y: 0.0,
                z: 16.0.inches().to::<Meters>() / 2.0,
            },
            Point {
                x: horizontal_offset.to::<Meters>(),
                y: 0.0,
                z: -16.0.inches().to::<Meters>() / 2.0,
            },
            30,
        )
    ).chain(
        line(
            Point {
                x: horizontal_offset.to::<Meters>(),
                y: 0.0,
                z: -vertical_offset.to::<Meters>(),
            },
            Point {
                x: 7.0.inches().to::<Meters>(),
                y: 0.0,
                z: -vertical_offset.to::<Meters>(),
            },
            12,
        )
    )
    .chain(
        line(
            Point {
                x: -7.0.inches().to::<Meters>(),
                y: 0.0,
                z: -vertical_offset.to::<Meters>(),
            },
            Point {
                x: -5.0.inches().to::<Meters>(),
                y: 0.0,
                z: -vertical_offset.to::<Meters>(),
            },
            3,
        )
    ).chain(
        line(
            Point {
                x: -horizontal_offset.to::<Meters>(),
                y: 0.0,
                z: -16.0.inches().to::<Meters>() / 2.0,
            },
            Point {
                x: -horizontal_offset.to::<Meters>(),
                y: 0.0,
                z: 16.0.inches().to::<Meters>() / 2.0,
            },
            3,
        )
    )
}
