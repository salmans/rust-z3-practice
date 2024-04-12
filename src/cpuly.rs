use z3::ast::Ast;
use z3::*;

const AMD: [u8; 128] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
    12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
    36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59,
    60, 61, 62, 63,
];

const NUMA: [[usize; 8]; 16] = [
    [0, 1, 2, 3, 64, 65, 66, 67],
    [4, 5, 6, 7, 68, 69, 70, 71],
    [8, 9, 10, 11, 72, 73, 74, 75],
    [12, 13, 14, 15, 76, 77, 78, 79],
    [16, 17, 18, 19, 80, 81, 82, 83],
    [20, 21, 22, 23, 84, 85, 86, 87],
    [24, 25, 26, 27, 88, 89, 90, 91],
    [28, 29, 30, 31, 92, 93, 94, 95],
    [32, 33, 34, 35, 96, 97, 98, 99],
    [36, 37, 38, 39, 100, 101, 102, 103],
    [40, 41, 42, 43, 104, 105, 106, 107],
    [44, 45, 46, 47, 108, 109, 110, 111],
    [48, 49, 50, 51, 112, 113, 114, 115],
    [52, 53, 54, 55, 116, 117, 118, 119],
    [56, 57, 58, 59, 120, 121, 122, 123],
    [60, 61, 62, 63, 124, 125, 126, 127],
];

const RANGE: usize = 128;

fn main() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);

    let mut cpus = Vec::new();
    for i in 0..RANGE {
        cpus.push(ast::Int::new_const(&ctx, format!("cpu_{}", i)));
    }

    for i in 0..RANGE {
        let is_zero = cpus[i].ge(&ast::Int::from_u64(&ctx, 0));
        let is_one = cpus[i].le(&ast::Int::from_u64(&ctx, 1));
        solver.assert(&ast::Bool::and(&ctx, &[&is_zero, &is_one]));
    }

    for i in 0..RANGE / 2 {
        for j in 0..RANGE {
            if i != j && AMD[i] == AMD[j] {
                solver.assert(
                    &ast::Int::add(&ctx, &[&cpus[i], &cpus[j]])
                        ._eq(&ast::Int::from_u64(&ctx, 1))
                        .not(),
                )
            }
        }
    }

    for n in NUMA {
        let elems = n.iter().map(|i| &cpus[*i]).collect::<Vec<_>>();
        let sum = ast::Int::add(&ctx, elems.as_slice());
        let le_4 = sum.le(&ast::Int::from_u64(&ctx, 4));
        let ge_2 = sum.ge(&ast::Int::from_u64(&ctx, 2));
        solver.assert(&ast::Bool::and(&ctx, &[&ge_2, &le_4]));
    }

    let elements = cpus.iter().collect::<Vec<_>>();
    let sum = ast::Int::add(&ctx, elements.as_slice());
    solver.assert(&sum._eq(&ast::Int::from_u64(&ctx, 52)));

    if solver.check() == SatResult::Sat {
        let model = solver.get_model();
        print_model(&model.unwrap(), &cpus);
    } else {
        println!("no solution")
    }
}

fn print_model(model: &Model, cpus: &[ast::Int]) {
    println!("work assignment:");
    for i in 0..RANGE {
        print!("{} ", get_cpu_work(model, cpus, i));
    }

    println!();
    println!(
        "h2o cpu count: {}",
        (0..RANGE).map(|i| get_cpu_work(model, cpus, i)).sum::<u8>()
    );

    for i in 0..RANGE / 2 {
        for j in 0..RANGE {
            if i != j && AMD[i] == AMD[j] {
                if get_cpu_work(model, cpus, i) != get_cpu_work(model, cpus, j) {
                    println!("siblings {}, {} share workload", i, j);
                }
            }
        }
    }

    for n in NUMA {
        let sum = n.iter().map(|i| get_cpu_work(model, cpus, *i)).sum::<u8>();
        if sum < 2 || sum > 4 {
            println!("bad NUMA zone assignment");
        }
    }
}

fn get_cpu_work(model: &Model, cpus: &[ast::Int], ind: usize) -> u8 {
    model
        .eval(cpus.get(ind).unwrap(), true)
        .unwrap()
        .as_u64()
        .unwrap() as u8
}
