mod finish_timeout;
pub use finish_timeout::*;

mod respawn_behaviour;
pub use respawn_behaviour::*;

mod warmup;
pub use warmup::*;

mod laps_number;
pub use laps_number::*;

mod maps;
pub use maps::*;

mod points_limit;
pub use points_limit::*;

mod rounds;
pub use rounds::*;

pub(crate) fn points_repartition_format(points: &Vec<i32>) -> String {
    let mut string = String::new();
    for point in points {
        string += &point.to_string();
        string += ","
    }
    string.trim_end_matches(",").to_string()
}
