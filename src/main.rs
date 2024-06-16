use entity::entity::{create_number, create_variable, Entity, MultiplicationFunction, SummationFunction, VariableTerm};

pub mod entity;

fn main() {
    let f = MultiplicationFunction::new(create_variable("x", 3), create_variable("x", 2));
    let df = f.differentiate();

    print!("\nOutput1: {}\n\n", df.to_str());
}
