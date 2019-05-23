use z3::*;
use std::collections::HashMap;

// Returns edges corresponding to a Hamiltonian cycle for the graph described by the matrix below:
const SIZE: usize = 4;
const GRAPH: [[i8; SIZE]; SIZE] = [
    [0, 1, 0, 0],
    [0, 0, 1, 0],
    [0, 0, 0, 1],
    [1, 1, 0, 0],
];

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
type PositionMap<'ctx> = HashMap<(usize, usize), Ast<'ctx>>;

fn main() {
    {
        println!("Finding a Hamiltonian cycle for:");
        let rows_to_string: Vec<String> = GRAPH.iter()
            .map(|r| format!("{:?}", r))
            .collect();
        println!("{}", rows_to_string.join("\n"));
        println!();
    }

    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);

    let pos = build_positions(&ctx);

    // Every node must take a position in the cycle:
    every_node(&solver, &pos);

    // Every node but the first position (also the last position) takes at most one position:
    no_duplicate_nodes(&solver, &pos);

    // Every position is assigned to a node:
    occupy_positions(&solver, &pos);

    // No two nodes are on the same position:
    one_position_per_node(&solver, &pos);

    // Two consecutive positions in the cycle must be adjacent nodes:
    adjacent_nodes(&solver, &pos);

    // The first node in the cycle must be the last node:
    form_cycle(&solver, &pos);


    print_solution(&solver, &pos);
}

fn build_positions(ctx: &Context) -> PositionMap {
    let mut map = HashMap::new();

    for i in 0..=SIZE {
        for j in 0..SIZE {
            let name = format!("p_{}_{}", i, j);
            let constant = ctx.named_bool_const(name.as_str());
            map.insert((i, j), constant);
        }
    }
    map
}

fn every_node(solver: &Solver, pos: &PositionMap) {
    for j in 0..SIZE {
        let mut formula = pos.get(&(0, j)).unwrap().clone();
        for i in 1..=SIZE {
            formula = formula.or(&[&pos.get(&(i, j)).unwrap()]);
        }
        solver.assert(&formula);
    }
}

fn no_duplicate_nodes(solver: &Solver, pos: &PositionMap) {
    for j in 0..SIZE {
        for i in 0..=SIZE {
            for k in (i + 1)..=SIZE {
                if i == 0 && k == SIZE {
                    continue;
                }
                let first = pos.get(&(i, j)).unwrap();
                let second = pos.get(&(k, j)).unwrap();
                solver.assert(&first.and(&[second]).not());
            }
        }
    }
}

fn occupy_positions(solver: &Solver, pos: &PositionMap) {
    for i in 0..=SIZE {
        let mut formula = pos.get(&(i, 0)).unwrap().clone();
        for j in 1..SIZE {
            formula = formula.or(&[&pos.get(&(i, j)).unwrap()]);
        }
        solver.assert(&formula);
    }
}

fn one_position_per_node(solver: &Solver, pos: &PositionMap) {
    for i in 0..=SIZE {
        for j in 0..SIZE {
            for k in (j + 1)..SIZE {
                let first = pos.get(&(i, j)).unwrap();
                let second = pos.get(&(i, k)).unwrap();
                solver.assert(&first.and(&[second]).not());
            }
        }
    }
}

fn adjacent_nodes(solver: &Solver, pos: &PositionMap) {
    for i in 0..SIZE {
        for j in 0..SIZE {
            for k in 0..SIZE {
                if GRAPH[j][k] == 0 {
                    let first = pos.get(&(i, j)).unwrap();
                    let second = pos.get(&(i + 1, k)).unwrap();
                    solver.assert(&first.and(&[second]).not());
                }
            }
        }
    }
}

fn form_cycle(solver: &Solver, pos: &PositionMap) {
    for j in 0..SIZE {
        let first = pos.get(&(0, j)).unwrap();
        let second = pos.get(&(SIZE, j)).unwrap();
        let both = first.and(&[&second]);
        let neither = first.not().and(&[&second.not()]);
        solver.assert(&both.or(&[&neither]))
    }
}

fn print_solution(solver: &Solver, graph: &PositionMap) {
    if solver.check() {
        let model = solver.get_model();
        let mut nodes: Vec<String> = Vec::new();

        for i in 0..=SIZE {
            for j in 0..SIZE {
                if model.eval(graph.get(&(i, j)).unwrap()).unwrap().as_bool().unwrap() {
                    nodes.push(j.to_string());
                }
            }
        }
        println!("{}", nodes.join(" -> "))
    } else {
        println!("There is no Hamiltonian cycle.")
    }
}