use z3::*;
use std::collections::HashMap;

const SIZE: i32 = 8;

type ConstantMap<'ctx> = HashMap<(i32, i32), Ast<'ctx>>;

fn main() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);

    let constants = build_constants(&ctx);

    // Each row has a queen:
    check_some_in_each_row(&solver, &constants);

    // Each row has at most one queen:
    check_one_in_each_row(&solver, &constants);

    // Each column has a queen:
    check_some_in_each_col(&solver, &constants);

    // Each column has at most one queen:
    check_one_in_each_col(&solver, &constants);

    // Check diagonally (top right to bottom left):
    check_slashes(&solver, &constants);

    // Check diagonally (top left to bottom right):
    check_backslashes(&solver, &constants);

    print_solution(&solver, &constants);
}

fn build_constants(ctx: &Context) -> ConstantMap {
    let mut map = HashMap::new();

    for i in 0..SIZE {
        for j in 0..SIZE {
            let cell = format!("c_{}_{}", i, j);
            map.insert((i, j), ctx.named_bool_const(cell.as_str()));
        }
    }

    return map;
}

fn check_some_in_each_row(solver: &Solver, constants: &ConstantMap) {
    for i in 0..SIZE {
        let mut formula = constants.get(&(i, 0)).unwrap().clone();
        for j in 1..SIZE {
            formula = formula.or(&[constants.get(&(i, j)).unwrap()]);
        }
        solver.assert(&formula);
    }
}

fn check_one_in_each_row(solver: &Solver, constants: &ConstantMap) {
    for i in 0..SIZE {
        for j in 0..SIZE {
            for k in (j + 1)..SIZE {
                let first = constants.get(&(i, j)).unwrap();
                let second = constants.get(&(i, k)).unwrap();
                solver.assert(&first.and(&[second]).not());
            }
        }
    }
}

fn check_some_in_each_col(solver: &Solver, constants: &ConstantMap) {
    for j in 0..SIZE {
        let mut formula = constants.get(&(0, j)).unwrap().clone();
        for i in 1..SIZE {
            formula = formula.or(&[constants.get(&(i, j)).unwrap()]);
        }
        solver.assert(&formula);
    }
}

fn check_one_in_each_col(solver: &Solver, constants: &ConstantMap) {
    for j in 0..SIZE {
        for i in 0..SIZE {
            for k in (i + 1)..SIZE {
                let first = constants.get(&(i, j)).unwrap();
                let second = constants.get(&(k, j)).unwrap();
                solver.assert(&first.and(&[second]).not());
            }
        }
    }
}

fn check_slashes(solver: &Solver, constants: &ConstantMap) {
    for i in 1..SIZE {
        for j in 0..(i + 1) {
            for k in (j + 1)..(i + 1) {
                let first = constants.get(&(j, i - j)).unwrap();
                let second = constants.get(&(k, i - k)).unwrap();

                solver.assert(&first.and(&[second]).not());
            }
        }
    }

    for i in 0..(SIZE - 2) {
        for j in 0..(SIZE - i - 1) {
            for k in (j + 1)..(SIZE - i - 1) {
                let first = constants.get(&(SIZE - j - 1, i + j + 1)).unwrap();
                let second = constants.get(&(SIZE - k - 1, i + k + 1)).unwrap();

                solver.assert(&first.and(&[second]).not());
            }
        }
    }
}

fn check_backslashes(solver: &Solver, constants: &ConstantMap) {
    for i in 0..(SIZE - 1) {
        for j in 0..(SIZE - i) {
            for k in (j + 1)..(SIZE - i) {
                let first = constants.get(&(i + j, j)).unwrap();
                let second = constants.get(&(i + k, k)).unwrap();

                solver.assert(&first.and(&[second]).not());
            }
        }
    }
    for i in 0..(SIZE - 1) {
        for j in 0..(SIZE - i) {
            for k in (j + 1)..(SIZE - i) {
                let first = constants.get(&(j, i + j)).unwrap();
                let second = constants.get(&(k, i + k)).unwrap();

                solver.assert(&first.and(&[second]).not());
            }
        }
    }
}

fn print_solution(solver: &Solver, constants: &ConstantMap) {
    solver.check();
    let model = solver.get_model();

    println!();
    for i in 0..SIZE {
        for j in 0..SIZE {
            let value = model.eval(constants.get(&(i, j)).unwrap()).unwrap()
                .as_bool().unwrap();
            if value {
                print!(" X ");
            } else {
                print!(" - ");
            }
        }
        println!();
    }
}