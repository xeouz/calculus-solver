use entity::entity::{create_variable, Entity, SummationFunction, VariableTerm};

pub mod entity;

fn main() {
    let f = SummationFunction::new(vec![create_variable("x".to_string(), 3), create_variable("x".to_string(), 2)]);
    let df = f.differentiate();

    print!("\nOutput: {}\n\n", df.to_str());
}
