use auto_ops::impl_op_ex;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Heading {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vector {
    pub x: i32,
    pub y: i32,
}

impl Vector {
    pub fn new(x: i32, y: i32) -> Vector {
        Vector { x, y }
    }

    pub fn len(&self) -> f64 {
        let xf: f64 = self.x.into();
        let yf: f64 = self.y.into();
        (xf.powf(2.0) + yf.powf(2.0)).sqrt()
    }

    pub fn unit(heading: Heading) -> Vector {
        match heading {
            Heading::North => Vector::new(0, -1),
            Heading::East => Vector::new(1, 0),
            Heading::South => Vector::new(0, 1),
            Heading::West => Vector::new(-1, 0),
        }
    }
}

impl_op_ex!(+ |lhs: &Vector, rhs: &Vector| -> Vector 
    { Vector { x: lhs.x + rhs.x, y: lhs.y + rhs.y } }
);

impl_op_ex!(+= |lhs: &mut Vector, rhs: &Vector| { 
    lhs.x += rhs.x; 
    lhs.y += rhs.y;
});
