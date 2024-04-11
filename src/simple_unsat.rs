use z3::*;

/// Simple test to check an unsatisfiable problem:
/// x & !y
/// !x & y
/// x & z
fn main() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);

    // define constants:
    let x = ast::Bool::new_const(&ctx, "x");
    let y = ast::Bool::new_const(&ctx, "y");
    let z = ast::Bool::new_const(&ctx, "z");

    let not_y = y.not(); // !y
    let x_and_not_y = ast::Bool::and(&ctx, &[&x, &not_y]); // x & !y
    let not_x = x.not(); // !x
    let not_x_or_y = ast::Bool::and(&ctx, &[&not_x, &y]); // !x & y
    let x_and_z = ast::Bool::and(&ctx, &[&x, &z]); // x & z

    solver.assert(&x_and_not_y);
    solver.assert(&not_x_or_y);
    solver.assert(&x_and_z);

    println!();
    println!("finding a model for:");
    eprintln!();
    println!("{}", solver);

    if solver.check() == SatResult::Unsat {
        print!("unsatisfiable problem")
    }
}
