use miniquad::rand;

pub trait RandomRange {
    fn gen_range(low: Self, high: Self) -> Self;
}

impl RandomRange for f32 {
    fn gen_range(low: Self, high: Self) -> Self {
        let r = unsafe { rand() } as f32 / miniquad::RAND_MAX as f32;
        low + (high - low) * r
    }
}
impl RandomRange for i32 {
    fn gen_range(low: i32, high: i32) -> Self {
        let r = unsafe { rand() } as f32 / miniquad::RAND_MAX as f32;
        let r = low as f32 + (high as f32 - low as f32) * r;
        r as i32
    }
}
impl RandomRange for i16 {
    fn gen_range(low: i16, high: i16) -> Self {
        let r = unsafe { rand() } as f32 / miniquad::RAND_MAX as f32;
        let r = low as f32 + (high as f32 - low as f32) * r;
        r as i16
    }
}

impl RandomRange for usize {
    fn gen_range(low: usize, high: usize) -> Self {
        let r = unsafe { rand() } as f32 / miniquad::RAND_MAX as f32;
        let r = low as f32 + (high as f32 - low as f32) * r;
        r as usize
    }
}

pub fn gen_range<T>(low: T, high: T) -> T
where
    T: RandomRange,
{
    T::gen_range(low, high)
}

pub trait ChooseRandom<T> {
    fn choose(&mut self) -> Option<&mut T>;
}

impl<T> ChooseRandom<T> for Vec<T> {
    fn choose(&mut self) -> Option<&mut T> {
        let ix = gen_range(0, self.len());
        self.get_mut(ix)
    }
}
