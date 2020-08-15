//! Geometric axis-related utilities. 

use vek::*;


/// Negativeness of a number. Enum over positive and negative. 
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum Sign {
    /// Positive. 
    Pos,
    /// Negative. 
    Neg,
}

impl Sign {
    /// Map `Pos` to `1` and `Neg` to `-1`. 
    pub fn to_i32(self) -> i32 {
        match self {
            Sign::Pos => 1,
            Sign::Neg => -1,
        }
    }
}


/// Three dimensional axis. Enum over X, Y, Z. 
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum Axis3 {
    X,
    Y,
    Z,
}

impl Axis3 {
    /// Convert to a `Vek3<i32>` positive unit vector on `self`'s axis. 
    pub fn to_unit_vec(self) -> Vec3<i32> {
        match self {
            Axis3::X => Vec3::new(1, 0, 0),
            Axis3::Y => Vec3::new(0, 1, 0),
            Axis3::Z => Vec3::new(0, 0, 1),
        }
    }
}


/// Three-dimensional axis-aligned unit vector (6 possible variants).  
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct AxisUnit3 {
    axis: Axis3,
    sign: Sign,
}

impl AxisUnit3 {
    /// Positive X direction. 
    pub const POSX: AxisUnit3 = AxisUnit3::new(Axis3::X, Sign::Pos);
    /// Positive Y direction. 
    pub const POSY: AxisUnit3 = AxisUnit3::new(Axis3::Y, Sign::Pos);
    /// Positive Z direction. 
    pub const POSZ: AxisUnit3 = AxisUnit3::new(Axis3::Z, Sign::Pos);
    /// Negative X direction. 
    pub const NEGX: AxisUnit3 = AxisUnit3::new(Axis3::X, Sign::Neg);
    /// Negative Y direction. 
    pub const NEGY: AxisUnit3 = AxisUnit3::new(Axis3::Y, Sign::Neg);
    /// Negative Z direction. 
    pub const NEGZ: AxisUnit3 = AxisUnit3::new(Axis3::Z, Sign::Neg);

    /// Alternate name for +X.
    pub const EAST: AxisUnit3 = AxisUnit3::new(Axis3::X, Sign::Pos);
    /// Alternate name for +Y.
    pub const UP: AxisUnit3 = AxisUnit3::new(Axis3::Y, Sign::Pos);
    /// Alternate name for +Z.
    pub const NORTH: AxisUnit3 = AxisUnit3::new(Axis3::Z, Sign::Pos);
    /// Alternate name for -X.
    pub const WEST: AxisUnit3 = AxisUnit3::new(Axis3::X, Sign::Neg);
    /// Alternate name for -Y.
    pub const DOWN: AxisUnit3 = AxisUnit3::new(Axis3::Y, Sign::Neg);
    /// Alternate name for -Z.
    pub const SOUTH: AxisUnit3 = AxisUnit3::new(Axis3::Z, Sign::Neg);

    /// Construct a new `AxisUnit3`.
    pub const fn new(axis: Axis3, sign: Sign) -> AxisUnit3 {
        AxisUnit3 { axis, sign }
    }

    /// Convert to a `Vek3<i32>`. 
    pub fn to_vec(self) -> Vec3<i32> {
        self.axis.to_unit_vec() * self.sign.to_i32()
    }

    /// Convert to an integer in [0, 6). 
    pub fn to_index(self) -> usize {
        match self {
            AxisUnit3 { axis: Axis3::X, sign: Sign::Pos } => 0,
            AxisUnit3 { axis: Axis3::Y, sign: Sign::Pos } => 1,
            AxisUnit3 { axis: Axis3::Z, sign: Sign::Pos } => 2,
            AxisUnit3 { axis: Axis3::X, sign: Sign::Neg } => 3,
            AxisUnit3 { axis: Axis3::Y, sign: Sign::Neg } => 4,
            AxisUnit3 { axis: Axis3::Z, sign: Sign::Neg } => 5,
        }
    }

    /// Convert from an integer in [0, 6). 
    ///
    /// Out-of-range index causes panic. 
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => AxisUnit3 { axis: Axis3::X, sign: Sign::Pos },
            1 => AxisUnit3 { axis: Axis3::Y, sign: Sign::Pos },
            2 => AxisUnit3 { axis: Axis3::Z, sign: Sign::Pos },
            3 => AxisUnit3 { axis: Axis3::X, sign: Sign::Neg },
            4 => AxisUnit3 { axis: Axis3::Y, sign: Sign::Neg },
            5 => AxisUnit3 { axis: Axis3::Z, sign: Sign::Neg },
            _ => panic!("index must be in [0, 6): {}", index),
        }
    }
}

/// Two dimensional axis. Enum over X, Y. 
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub enum Axis2 {
    X,
    Y,
}

impl Axis2 {
    /// Convert to a `Vek2<i32>` positive unit vector on `self`'s axis. 
    pub fn to_unit_vec(self) -> Vec2<i32> {
        match self {
            Axis2::X => Vec2::new(1, 0),
            Axis2::Y => Vec2::new(0, 1),
        }
    }
}


/// Two-dimensional axis-aligned unit vector (4 possible variants).  
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
pub struct AxisUnit2 {
    axis: Axis2,
    sign: Sign,
}

impl AxisUnit2 {
    /// Positive X direction. 
    pub const POSX: AxisUnit2 = AxisUnit2::new(Axis2::X, Sign::Pos);
    /// Positive Y direction. 
    pub const POSY: AxisUnit2 = AxisUnit2::new(Axis2::Y, Sign::Pos);
    /// Negative X direction. 
    pub const NEGX: AxisUnit2 = AxisUnit2::new(Axis2::X, Sign::Neg);
    /// Negative Y direction. 
    pub const NEGY: AxisUnit2 = AxisUnit2::new(Axis2::Y, Sign::Neg);

    /// Alternate name for +X.
    pub const EAST: AxisUnit2 = AxisUnit2::new(Axis2::X, Sign::Pos);
    /// Alternate name for +Y.
    pub const NORTH: AxisUnit2 = AxisUnit2::new(Axis2::Y, Sign::Pos);
    /// Alternate name for -X.
    pub const WEST: AxisUnit2 = AxisUnit2::new(Axis2::X, Sign::Neg);
    /// Alternate name for -Y.
    pub const SOUTH: AxisUnit2 = AxisUnit2::new(Axis2::Y, Sign::Neg);

    /// Construct a new `AxisUnit2`.
    pub const fn new(axis: Axis2, sign: Sign) -> AxisUnit2 {
        AxisUnit2 { axis, sign }
    }

    /// Convert to a `Vek2<i32>`. 
    pub fn to_vec(self) -> Vec2<i32> {
        self.axis.to_unit_vec() * self.sign.to_i32()
    }

     /// Convert to an integer in [0, 4). 
    pub fn to_index(self) -> usize {
        match self {
            AxisUnit2 { axis: Axis2::X, sign: Sign::Pos } => 0,
            AxisUnit2 { axis: Axis2::Y, sign: Sign::Pos } => 1,
            AxisUnit2 { axis: Axis2::X, sign: Sign::Neg } => 2,
            AxisUnit2 { axis: Axis2::Y, sign: Sign::Neg } => 3,
        }
    }

    /// Convert from an integer in [0, 4). 
    ///
    /// Out-of-range index causes panic. 
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => AxisUnit2 { axis: Axis2::X, sign: Sign::Pos },
            1 => AxisUnit2 { axis: Axis2::Y, sign: Sign::Pos },
            2 => AxisUnit2 { axis: Axis2::X, sign: Sign::Neg },
            3 => AxisUnit2 { axis: Axis2::Y, sign: Sign::Neg },
            _ => panic!("index must be in [0, 4): {}", index),
        }
    }
}