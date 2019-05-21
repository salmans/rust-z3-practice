use z3::*;

/// Simple test to get a solution for a satisfiable problem:
/// x -> y
/// y -> x
fn main() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);

    // define constants:
    let x = ctx.named_bool_const("x");
    let y = ctx.named_bool_const("y");

    let not_y = y.not();                    // !y
    let x_or_not_y = x.or(&[&not_y]); // y -> x
    let not_x = x.not();                    // !x
    let not_x_or_y = not_x.or(&[&y]); // x -> y

    solver.assert(&x_or_not_y);
    solver.assert(&not_x_or_y);

    println!("finding a solution for:");
    println!("{}", solver);

    if solver.check() {
        let model = solver.get_model();
        println!("solution:");
        println!("x -> {}", model.eval(&x).unwrap().as_bool().unwrap());
        println!("y -> {}", model.eval(&y).unwrap().as_bool().unwrap());
    }
}
