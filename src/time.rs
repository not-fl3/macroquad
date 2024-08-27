pub struct Time {
    pub start_time: f64,
    pub frame_time: f64,
    pub last_frame_time: f64,
}

impl Time {
    pub fn new() -> Time {
        let start_time = miniquad::date::now();
        let last_frame_time = miniquad::date::now();
        let frame_time = 0.0;

        Time {
            start_time,
            frame_time,
            last_frame_time,
        }
    }

    pub fn time_since_start(&self) -> f32 {
        (miniquad::date::now() - self.start_time) as f32
    }

    pub fn update(&mut self) {
        self.frame_time = miniquad::date::now() - self.last_frame_time;
        self.last_frame_time = miniquad::date::now();
    }
}
