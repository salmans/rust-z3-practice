use std::collections::HashMap;
use z3::ast::Bool;
use z3::*;

// Returns edges corresponding to a Hamiltonian cycle for the graph described by the matrix below:
const SIZE: usize = 4;
const GRAPH: [[i8; SIZE]; SIZE] = [[0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1], [1, 1, 0, 0]];

//const SIZE: usize = 4;
//const GRAPH: [[i8; SIZE]; SIZE] = [
//    [0, 1, 0, 0],
//    [0, 0, 1, 0],
//    [0, 0, 0, 1],
//    [0, 1, 0, 0],
//];

//const SIZE: usize = 5;
//const GRAPH: [[i8; SIZE]; SIZE] = [
//    [0, 1, 1, 1, 1],
//    [1, 0, 0, 1, 0],
//    [1, 0, 0, 1, 1],
//    [1, 1, 1, 0, 0],
//    [1, 0, 1, 0, 0],
//];

// Maps the position of a node in the Hamiltonian cycle to the node.
type PositionMap<'ctx> = HashMap<(usize, usize), Bool<'ctx>>;

fn main() {
    {
        println!("Finding a Hamiltonian cycle for:");
        let rows_to_string: Vec<String> = GRAPH.iter().map(|r| format!("{:?}", r)).collect();
        println!("{}", rows_to_string.join("\n"));
        println!();
    }

    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);

    let pos = build_positions(&ctx);

    // Every node must take a position in the cycle:
    every_node(&ctx, &solver, &pos);

    // Every node but the first position (also the last position) takes at most one position:
    no_duplicate_nodes(&ctx, &solver, &pos);

    // Every position is assigned to a node:
    occupy_positions(&ctx, &solver, &pos);

    // No two nodes are on the same position:
    one_position_per_node(&ctx, &solver, &pos);

    // Two consecutive positions in the cycle must be adjacent nodes:
    adjacent_nodes(&ctx, &solver, &pos);

    // The first node in the cycle must be the last node:
    form_cycle(&ctx, &solver, &pos);

    if solver.check() == SatResult::Sat {
        let model = solver.get_model();
        print_model(&model.unwrap(), &pos);
    } else {
        println!("There is no Hamiltonian cycle.")
    }
}

fn build_positions(ctx: &Context) -> PositionMap {
    let mut map = HashMap::new();

    for i in 0..=SIZE {
        for j in 0..SIZE {
            let name = format!("p_{}_{}", i, j);
            let constant = ast::Bool::new_const(&ctx, name.as_str());
            map.insert((i, j), constant);
        }
    }
    map
}

fn every_node(ctx: &Context, solver: &Solver, pos: &PositionMap) {
    for j in 0..SIZE {
        let mut formula = pos.get(&(0, j)).unwrap().clone();
        for i in 1..=SIZE {
            formula = ast::Bool::or(&ctx, &[&formula, &pos.get(&(i, j)).unwrap()]);
        }
        solver.assert(&formula);
    }
}

fn no_duplicate_nodes(ctx: &Context, solver: &Solver, pos: &PositionMap) {
    for j in 0..SIZE {
        for i in 0..=SIZE {
            for k in (i + 1)..=SIZE {
                if i == 0 && k == SIZE {
                    continue;
                }
                let first = pos.get(&(i, j)).unwrap();
                let second = pos.get(&(k, j)).unwrap();
                solver.assert(&ast::Bool::and(ctx, &[first, second]).not());
            }
        }
    }
}

fn occupy_positions(ctx: &Context, solver: &Solver, pos: &PositionMap) {
    for i in 0..=SIZE {
        let mut formula = pos.get(&(i, 0)).unwrap().clone();
        for j in 1..SIZE {
            formula = ast::Bool::or(ctx, &[&formula, &pos.get(&(i, j)).unwrap()]);
        }
        solver.assert(&formula);
    }
}

fn one_position_per_node(ctx: &Context, solver: &Solver, pos: &PositionMap) {
    for i in 0..=SIZE {
        for j in 0..SIZE {
            for k in (j + 1)..SIZE {
                let first = pos.get(&(i, j)).unwrap();
                let second = pos.get(&(i, k)).unwrap();
                solver.assert(&ast::Bool::and(ctx, &[first, second]).not());
            }
        }
    }
}

fn adjacent_nodes(ctx: &Context, solver: &Solver, pos: &PositionMap) {
    for i in 0..SIZE {
        for j in 0..SIZE {
            for k in 0..SIZE {
                if GRAPH[j][k] == 0 {
                    let first = pos.get(&(i, j)).unwrap();
                    let second = pos.get(&(i + 1, k)).unwrap();
                    solver.assert(&ast::Bool::and(ctx, &[first, second]).not());
                }
            }
        }
    }
}

fn form_cycle(ctx: &Context, solver: &Solver, pos: &PositionMap) {
    for j in 0..SIZE {
        let first = pos.get(&(0, j)).unwrap();
        let second = pos.get(&(SIZE, j)).unwrap();
        let both = ast::Bool::and(ctx, &[first, second]);
        let neither = ast::Bool::and(ctx, &[&first.not(), &second.not()]);
        solver.assert(&ast::Bool::or(ctx, &[&both, &neither]))
    }
}

fn print_model(model: &Model, graph: &PositionMap) {
    let mut nodes: Vec<String> = Vec::new();

    for i in 0..=SIZE {
        for j in 0..SIZE {
            if model
                .eval(graph.get(&(i, j)).unwrap(), false)
                .unwrap()
                .as_bool()
                .unwrap()
            {
                nodes.push(j.to_string());
            }
        }
    }
    println!("{}", nodes.join(" -> "))
}
