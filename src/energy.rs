use std::ops::{Add, Mul, Sub};

#[derive(Copy, Clone, Debug)]
pub struct Energy {
    red: i16,
    green: i16,
    white: i16,
    blue: i16,
    black: i16,
    total: i16,
}

impl Energy {
    fn new(red: i16, green: i16, white: i16, blue: i16, black: i16, total: i16) -> Self {
        Energy {
            red,
            green,
            white,
            blue,
            black,
            total,
        }
    }

    fn red(red: i16) -> Self {
        Self::new(red, 0, 0, 0, 0, red)
    }

    fn green(green: i16) -> Self {
        Self::new(0, green, 0, 0, 0, green)
    }

    fn white(white: i16) -> Self {
        Self::new(0, 0, white, 0, 0, white)
    }

    fn blue(blue: i16) -> Self {
        Self::new(0, 0, 0, blue, 0, blue)
    }

    fn black(black: i16) -> Self {
        Self::new(0, 0, 0, 0, black, black)
    }

    fn neutral(total: i16) -> Self {
        Self::new(0, 0, 0, 0, 0, total)
    }

    fn is_valid(&self) -> bool {
        self.red >= 0
            && self.green >= 0
            && self.white >= 0
            && self.blue >= 0
            && self.black >= 0
            && self.total >= 0
    }
}

impl Add for Energy {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(
            self.red + other.red,
            self.green + other.green,
            self.white + other.white,
            self.blue + other.blue,
            self.black + other.black,
            self.total + other.total,
        )
    }
}

impl Sub for Energy {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(
            self.red - other.red,
            self.green - other.green,
            self.white - other.white,
            self.blue - other.blue,
            self.black - other.black,
            self.total - other.total,
        )
    }
}

impl Mul<i16> for Energy {
    type Output = Self;

    fn mul(self, other: i16) -> Self {
        Self::new(
            self.red * other,
            self.green * other,
            self.white * other,
            self.blue * other,
            self.black * other,
            self.total * other,
        )
    }
}

impl Mul<Energy> for i16 {
    type Output = Energy;
    fn mul(self, other: Energy) -> Energy {
        other * self
    }
}
