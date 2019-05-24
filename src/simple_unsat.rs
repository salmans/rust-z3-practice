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
    let x = ctx.named_bool_const("x");
    let y = ctx.named_bool_const("y");
    let z = ctx.named_bool_const("z");

    let not_y = y.not();                     // !y
    let x_or_not_y = x.and(&[&not_y]); // x & !y
    let not_x = x.not();                     // !x
    let not_x_or_y = not_x.and(&[&y]); // !x & y
    let x_and_z = x.and(&[&z]);        // x & z

    solver.assert(&x_or_not_y);
    solver.assert(&not_x_or_y);
    solver.assert(&x_and_z);

    println!();
    println!("finding a model for:");
    eprintln!();
    println!("{}", solver);

    if !solver.check() {
        print!("unsatisfiable problem")
    }
}
