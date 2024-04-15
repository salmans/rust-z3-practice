use z3::ast::Ast;
use z3::*;

// Assignment of CPUs to cores (index to core ID)
const AMD: [u8; 128] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
    12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
    36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59,
    60, 61, 62, 63,
];

// Assignment of CPUs to NUMA zones (CPU ID to NUMA zone index)
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

// Total number of CPUs
const CPU_COUNT: usize = 128;
const VARNISH: u8 = 0;
const H2O: u8 = 1;

fn main() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);
    solver.push();

    // Keep CPU costant names in a vector to refer to them by index.
    let mut cpus = Vec::new();
    for i in 0..CPU_COUNT {
        cpus.push(ast::Int::new_const(&ctx, format!("cpu_{}", i)));
    }

    // Values assigned to CPUs are either VARNISH (0) or H2O (1)
    for i in 0..CPU_COUNT {
        let is_varnish = cpus[i]._eq(&ast::Int::from_u64(&ctx, VARNISH as u64));
        let is_h2o = cpus[i]._eq(&ast::Int::from_u64(&ctx, H2O as u64));
        solver.assert(&ast::Bool::or(&ctx, &[&is_varnish, &is_h2o]));
    }

    // H2O cannot share siblings (CPUs assigned to the same core).
    for i in 0..CPU_COUNT / 2 {
        for j in 0..CPU_COUNT {
            if i != j && AMD[i] == AMD[j] {
                solver.assert(
                    &ast::Int::add(&ctx, &[&cpus[i], &cpus[j]])
                        ._eq(&ast::Int::from_u64(&ctx, H2O as u64))
                        .not(),
                )
            }
        }
    }

    // In each NUMA zone, at least 2 CPUs are assigned to H2O.
    for n in NUMA {
        let elems = n.iter().map(|i| &cpus[*i]).collect::<Vec<_>>();
        let sum = ast::Int::add(&ctx, elems.as_slice());
        let ge_2 = sum.ge(&ast::Int::from_u64(&ctx, 2 * H2O as u64));
        solver.assert(&ge_2);
    }

    // In each NUMA zone, at least 2 and at most 4 CPUs are assigned to H2O.
    // for n in NUMA {
    //     let elems = n.iter().map(|i| &cpus[*i]).collect::<Vec<_>>();
    //     let sum = ast::Int::add(&ctx, elems.as_slice());
    //     let le_4 = sum.le(&ast::Int::from_u64(&ctx, 4 * H2O as u64));
    //     let ge_2 = sum.ge(&ast::Int::from_u64(&ctx, 2 * H2O as u64));
    //     solver.assert(&ast::Bool::or(&ctx, &[&ge_2, &le_4]));
    // }

    // The total number of CPUs assigned to H2O is 52
    let elements = cpus.iter().collect::<Vec<_>>();
    let sum = ast::Int::add(&ctx, elements.as_slice());
    solver.assert(&sum._eq(&ast::Int::from_u64(&ctx, 52 * H2O as u64)));

    if solver.check() == SatResult::Sat {
        let mut model = solver.get_model().unwrap();
        print_model(&model, &cpus);

        // Optimize while there's an answer:
        while let Some(m) = optimize(&ctx, &solver, model, &cpus) {
            print_model(&m, &cpus);
            model = m;
        }
    } else {
        println!("no solution")
    }
}

// Given an existing solution, tries to find a new solution where the maximum
// number of CPUs assigned to H2O in all NUMA zones is (strinctly) less than
// the maximum number of CPUs assigned to NUMA zones of the existing solution.
fn optimize<'ctx>(
    ctx: &Context,
    solver: &'ctx Solver,
    model: Model,
    cpus: &[ast::Int],
) -> Option<Model<'ctx>> {
    // Find the maximum number of CPUs in a NUMA zone.
    let mut max = 0;
    for n in NUMA {
        let sum = n.iter().map(|i| get_cpu_work(&model, cpus, *i)).sum::<u8>();
        if sum > max {
            max = sum;
        }
    }

    solver.push(); // Save the existing constraints.

    // Ask for a new solution where every NUMA zone is assigned to a number
    // of H2O processes, less than `max`.
    for n in NUMA {
        let elems = n.iter().map(|i| &cpus[*i]).collect::<Vec<_>>();
        let sum = ast::Int::add(&ctx, elems.as_slice());
        let less = sum.lt(&ast::Int::from_u64(&ctx, max as u64));
        solver.assert(&less);
    }

    if solver.check() == SatResult::Sat {
        let model = solver.get_model();
        solver.pop(1); // Backtrack (maybe for the next round of minimization).
        model
    } else {
        None
    }
}

fn print_model(model: &Model, cpus: &[ast::Int]) {
    println!("work assignment:");
    for i in 0..CPU_COUNT {
        print!("{} ", get_cpu_work(model, cpus, i));
    }

    println!();
    println!(
        "h2o cpu count: {}",
        (0..CPU_COUNT)
            .map(|i| get_cpu_work(model, cpus, i))
            .sum::<u8>()
    );

    for i in 0..CPU_COUNT / 2 {
        for j in 0..CPU_COUNT {
            if i != j && AMD[i] == AMD[j] {
                if get_cpu_work(model, cpus, i) != get_cpu_work(model, cpus, j) {
                    println!("siblings {}, {} share workload", i, j);
                }
            }
        }
    }

    for n in NUMA {
        let sum = n.iter().map(|i| get_cpu_work(model, cpus, *i)).sum::<u8>();
        println!("H2Os in NUMA zone: {}", sum);
    }
}

fn get_cpu_work(model: &Model, cpus: &[ast::Int], ind: usize) -> u8 {
    model
        .eval(cpus.get(ind).unwrap(), true)
        .unwrap()
        .as_u64()
        .unwrap() as u8
}
